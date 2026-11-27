//! The world-generation worker — `WorldGen.CreateNewWorld`'s
//! `Task.Factory.StartNew(worldGenCallback)` (`WorldGen.cs:6607-6625`) and
//! the `worldGenCallback` body (`:6591-6605`): the generation runs off the
//! frame thread while `UIWorldLoad` draws the progress, then the callback
//! saves (`WorldFile.SaveNewWorld`, `:6597` — only on the completed arm)
//! and the menu hops to 6 (`:6599-6602`), which the next `UpdateMenu`
//! turns into the world select (`Main.cs:49468-49472`).
//!
//! The progress/controller pair is the C#'s `WorldGenerator.
//! CurrentGenerationProgress` / `CurrentController` statics
//! (`WorldGenerator.cs:398-400`): the port parks them on the one live job
//! the App owns (`App.world_gen_job`). The worker implements
//! [`tr_core::worldgen::pipeline::GenProgressHook`] over the shared
//! `GenerationProgress` (the S1-E-17 seam); `QueuedAbort` is the atomic the
//! loading screen's Inventory trigger sets (`UIWorldLoad.cs:54-58`).

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tr_core::generation_progress::GenerationProgress;
use tr_core::worldgen::pipeline::{GenOutcome, GenProgressHook};

use crate::ui::world_creation::{self, NewWorldDraft};
use crate::ui::world_load::WorldLoadFlags;

/// The worker's outcome — `worldGenCallback`'s tail state
/// (`WorldGen.cs:6594-6604`): the saved path on the completed arm, the
/// abort flag (`GenerateWorld` returned false, `:6594-6598` — no save), or
/// the save error (the port's own arm; the vanilla `SaveNewWorld` failure
/// throws into the task).
#[derive(Clone, Debug)]
pub(crate) enum JobResult {
    /// `WorldFile.SaveNewWorld` succeeded (`:6597`).
    Saved(PathBuf),
    /// `QueuedAbort` broke the pass loop (`WorldGenerator.cs:433-437`);
    /// the partial world is discarded, exactly as the C# skips the save.
    Aborted,
    /// The save failed (the port surfaces the error through `front_hint`).
    Failed(String),
}

/// The shared progress snapshot the loading screen reads —
/// `UIWorldLoad.Draw`'s `_progress` latch (`UIWorldLoad.cs:69`, `:83-85`).
#[derive(Clone, Debug, Default)]
pub(crate) struct ProgressSnapshot {
    pub total_progress: f64,
    pub value: f64,
    pub message: String,
    /// The running step's display name (`Controller.CurrentPass.Name`,
    /// `UIWorldGenDebug.cs:245`) — "" between steps / before the first.
    /// The `UIWorldGenDebug` driver's `PassState.running` reads it
    /// (W4-f23 closeout item 1).
    pub current_step: String,
    /// The completed-step count (`WorldGen.Manifest.GenPassResults.Count`,
    /// `UIWorldGenDebug.cs:247`) — `PassState.completed` is `count > index`.
    pub completed_steps: usize,
}

/// The live generation job — the App's `WorldGenerator.CurrentController`
/// + `CurrentGenerationProgress` pair (`WorldGenerator.cs:398-400`) and
/// the `CreateNewWorld` task handle. One job at a time (the C# statics are
/// single-slot too).
pub(crate) struct WorldGenJob {
    /// `GenerationProgress` behind the worker/frame lock — the C# fields
    /// are plain statics the two threads read/write without a lock (a
    /// benign race in the original); the port's Mutex keeps the snapshot
    /// coherent without changing the math.
    progress: Arc<Mutex<GenerationProgress>>,
    /// `Controller.QueuedAbort` (`WorldGenerator.cs:80`).
    queued_abort: Arc<AtomicBool>,
    /// The task's result slot — `None` while the worker runs.
    result: Arc<Mutex<Option<JobResult>>>,
    /// The `WorldGen` seed statics the loading screen's message overrides
    /// read (`UIWorldLoad.cs:93-116`) — resolved from the draft's seed at
    /// start (`WorldGen.Reset`, `WorldGen.cs:11115-11138`).
    pub flags: WorldLoadFlags,
    /// `WorldGen.crimson` / `generatingRandomEvil`
    /// (`WorldGen.cs:11305-11316`) — the bar's colour pick
    /// (`UIGenProgressBar.cs:61-62`).
    pub crimson: bool,
    pub random_evil: bool,
    /// `Controller.CurrentPass.Name` (`UIWorldGenDebug.cs:245`) — the debug
    /// driver's running-step marker (W4-f23 closeout item 1).
    current_step: Arc<Mutex<String>>,
    /// `WorldGen.Manifest.GenPassResults.Count` (`UIWorldGenDebug.cs:247`).
    completed_steps: Arc<std::sync::atomic::AtomicUsize>,
    /// The draft's micro-seed flags — the debug driver's pass-list enable
    /// gate (`pass_disabled_by_special_seeds`, `WorldGen.cs:22493-22509`).
    pub micro_flags: tr_core::seeds::MicroSeedFlags,
}

/// The worker-side hook: `RunPass`'s `Start`/`End` bookends
/// (`WorldGenerator.cs:498`/`:507`) plus the pass body's `progress.Message`
/// assignment, resolved through the step message table
/// (`worldgen::progress::step_message` — the `WorldGen.cs` per-pass sites).
struct WorkerHook {
    progress: Arc<Mutex<GenerationProgress>>,
    queued_abort: Arc<AtomicBool>,
    /// `Controller.CurrentPass.Name` (`UIWorldGenDebug.cs:245`) — the running
    /// step's display name, "" between steps. The debug driver reads it.
    current_step: Arc<Mutex<String>>,
    /// `WorldGen.Manifest.GenPassResults.Count` (`UIWorldGenDebug.cs:247`) —
    /// the completed-step count, bumped by `end_step`.
    completed_steps: Arc<std::sync::atomic::AtomicUsize>,
}

impl GenProgressHook for WorkerHook {
    fn begin_generation(&mut self, total_weight: f64) {
        // `GenerateWorld`'s `TotalWeight` sum (`WorldGenerator.cs:424`).
        self.progress.lock().unwrap().total_weight = total_weight;
    }
    fn begin_step(&mut self, name: &str, weight: f64) {
        let mut p = self.progress.lock().unwrap();
        p.start(weight);
        if let Some(msg) = tr_core::worldgen::progress::step_message(name) {
            // The pass body's `progress.Message = Lang.gen[i].Value` /
            // `Language.GetTextValue(key)` — the language manager is the
            // process-global mutex, safe from the worker thread.
            p.set_message(&msg.resolve(&crate::lang::text));
        }
        // `Controller.CurrentPass = pass` (`WorldGenerator.cs:498`'s RunPass
        // head) — the debug driver's running-row marker.
        *self.current_step.lock().unwrap() = name.to_string();
    }
    fn end_step(&mut self) {
        self.progress.lock().unwrap().end();
        // `Controller.CurrentPass = null` + the GenPassResult append
        // (`WorldGenerator.cs:507`'s RunPass tail) — the row completes.
        self.current_step.lock().unwrap().clear();
        self.completed_steps.fetch_add(1, Ordering::Relaxed);
    }
    fn should_abort(&self) -> bool {
        self.queued_abort.load(Ordering::Relaxed)
    }
}

impl WorldGenJob {
    /// `WorldGen.CreateNewWorld` (`WorldGen.cs:6607-6625`) minus the
    /// `UIWorldLoad` SetState (the caller enters the screen): resolve the
    /// draft's generation options, spawn the `worldGenCallback` task.
    /// Returns `None` when the worlds dir cannot resolve (the port's
    /// `Main.WorldPath` stand-in, `save.rs:155-164`).
    pub(crate) fn start(draft: &NewWorldDraft) -> Option<Self> {
        let dir = tr_core::save::default_world_dir()?;
        let name = draft.name.trim().to_string();
        let (width, height) = world_creation::size_dimensions(draft.size);
        // W4-d85: `FinishCreatingWorld`'s seed join (`UIWorldCreation.cs:
        // 991-995`) — the entered micro-seed texts prepend the base seed.
        let seed_text = draft.effective_seed_text();
        let seed = world_creation::seed_value(&name, &seed_text);
        let (text_flags, micro_flags) = world_creation::seed_identity(&seed_text);
        // The Advanced tab's toggled options carry the classic flags the
        // seed text does not name (`WorldGenerationOptions`' live Enabled
        // states, `WorldGen.Reset()`'s read); a special seed TEXT takes
        // `SetSeedToRandomWithCurrentEvents` instead (`:983-985`) — the
        // port keeps the typed text's hash (the deterministic stand-in)
        // but the option flags still apply.
        let flags = text_flags | draft.option_flags();
        let crimson = match draft.evil {
            1 => false,
            2 => true,
            _ => tr_core::worldgen::roll_crimson(seed),
        };
        let opts = tr_core::worldgen::GenOptions {
            name: name.clone(),
            width,
            height,
            seed,
            crimson,
            // `_optionDifficulty` (`UIWorldCreation.cs:129`): Main.GameMode.
            game_mode: draft.difficulty as i32,
            flags,
            micro_flags,
        };
        // `WorldGen.Reset`'s derived statics (`WorldGen.cs:11115-11138`):
        // drunkWorldGenText = drunkWorldGen (`:11138`); the secret-seed
        // booleans ride the flag bits. `placingTraps` is a pass-runtime
        // static (`:18662`) the port's compact pipeline never sets — it
        // stays false (documented, ANALYSIS "The loading screen").
        let load_flags = WorldLoadFlags {
            drunk_world_gen: flags.contains(tr_core::seeds::SeedFlags::DRUNK),
            not_the_bees: flags.contains(tr_core::seeds::SeedFlags::NOT_THE_BEES),
            get_good_world_gen: flags.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
            no_traps_world_gen: flags.contains(tr_core::seeds::SeedFlags::NO_TRAPS),
            placing_traps: false,
            zenith_world: flags.contains(tr_core::seeds::SeedFlags::ZENITH),
            tenth_anniversary_world_gen: flags
                .contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY),
        };
        // `generatingRandomEvil` (`WorldGen.cs:11305-11316`): true only
        // for the Random evil choice (the draft's 0).
        let random_evil = draft.evil == 0;
        Some(Self::spawn(opts, dir.join(format!("{name}.trwld")), load_flags, crimson, random_evil))
    }

    /// The task body — `Task.Factory.StartNew(() => worldGenCallback(...))`
    /// (`WorldGen.cs:6621-6624`). Split out so the tests can drive a small
    /// world through the same path.
    pub(crate) fn spawn(
        opts: tr_core::worldgen::GenOptions,
        path: PathBuf,
        flags: WorldLoadFlags,
        crimson: bool,
        random_evil: bool,
    ) -> Self {
        Self::spawn_with_abort(opts, path, flags, crimson, random_evil, false)
    }

    /// [`Self::spawn`] with the abort flag pre-set (the tests' deterministic
    /// abort arm — the flag lands before the worker's first head poll).
    fn spawn_with_abort(
        opts: tr_core::worldgen::GenOptions,
        path: PathBuf,
        flags: WorldLoadFlags,
        crimson: bool,
        random_evil: bool,
        pre_abort: bool,
    ) -> Self {
        let progress = Arc::new(Mutex::new(GenerationProgress::new()));
        let queued_abort = Arc::new(AtomicBool::new(pre_abort));
        let result = Arc::new(Mutex::new(None));
        let current_step = Arc::new(Mutex::new(String::new()));
        let completed_steps = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let micro_flags = opts.micro_flags;
        let job = WorldGenJob {
            progress: progress.clone(),
            queued_abort: queued_abort.clone(),
            result: result.clone(),
            flags,
            crimson,
            random_evil,
            current_step: current_step.clone(),
            completed_steps: completed_steps.clone(),
            micro_flags,
        };
        let remix = opts.flags.contains(tr_core::seeds::SeedFlags::REMIX);
        let seed = opts.seed;
        std::thread::spawn(move || {
            // `worldGenCallback` (`WorldGen.cs:6591-6605`): GenerateWorld,
            // then SaveNewWorld on the completed arm only (`:6595-6598`).
            let mut hook = WorkerHook {
                progress,
                queued_abort,
                current_step,
                completed_steps,
            };
            let mut chest_stock = crate::chest_gen::chest_stock_hook(seed, remix);
            let (mut world, outcome) = tr_core::worldgen::generate_with_progress(
                &opts,
                Some(&mut chest_stock),
                &mut hook,
            );
            let r = match outcome {
                GenOutcome::Completed => {
                    // The header records the draft's seed (`create_world_in`'s
                    // `world.seed = seed`, the `save.rs` header write).
                    world.seed = seed;
                    // `WorldFile.CreateMetadata:600-604` + the WorldId roll
                    // (`WorldGen.cs:11337`) — the same identity stamp the
                    // synchronous `create_world_in` tail runs.
                    let mut identity = tr_core::wld::identity_rng();
                    tr_core::wld::stamp_created_identity(&mut world, &mut identity);
                    world.world_id = identity.next_max(i32::MAX);
                    let side = tr_core::save::WorldSide {
                        chests: world.chests.clone(),
                        // W4-g38 C5: the gen-time starter town NPCs ride the
                        // save's native section (the SpawnStarterNPCs pass).
                        npcs: world.starter_npcs.clone(),
                        ..Default::default()
                    };
                    match tr_core::save::save(&path, &world, &side) {
                        Ok(()) => JobResult::Saved(path),
                        Err(e) => JobResult::Failed(format!("{e:#}")),
                    }
                }
                // `:6595-6598` — the abort skips SaveNewWorld; the partial
                // world drops with the task.
                GenOutcome::Aborted => JobResult::Aborted,
            };
            *result.lock().unwrap() = Some(r);
        });
        job
    }

    /// `WorldGenerator.CurrentGenerationProgress`'s three drawn fields
    /// (`UIWorldLoad.cs:83-85`) — the frame's snapshot latch.
    pub(crate) fn snapshot(&self) -> ProgressSnapshot {
        let p = self.progress.lock().unwrap();
        ProgressSnapshot {
            total_progress: p.total_progress(),
            value: p.value(),
            message: p.message(),
            current_step: self.current_step.lock().unwrap().clone(),
            completed_steps: self.completed_steps.load(Ordering::Relaxed),
        }
    }

    /// `Controller.QueuedAbort = true` (`UIWorldLoad.cs:56`).
    pub(crate) fn queue_abort(&self) {
        self.queued_abort.store(true, Ordering::Relaxed);
    }

    /// `Controller.QueuedAbort`'s read (`UIWorldLoad.cs:54`, `:86`).
    pub(crate) fn queued_abort(&self) -> bool {
        self.queued_abort.load(Ordering::Relaxed)
    }

    /// The task's result, taken once — `None` while the worker runs.
    pub(crate) fn take_result(&self) -> Option<JobResult> {
        self.result.lock().unwrap().take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The worker runs the real pipeline through the hook and saves: the
    /// completed arm produces a loadable `.trwld` and the progress reaches
    /// 1.0 (`WorldGenerator.cs:424`'s weight sum + the Start/End bookends).
    #[test]
    fn s1_e_17_the_worker_generates_and_saves_a_world() {
        // The language lock: the worker reads `lang::text` (the step
        // messages); pin the culture against the tooltip tests.
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let dir = std::env::temp_dir().join(format!("tr-s1-e-17-job-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let opts = tr_core::worldgen::GenOptions {
            name: "job".into(),
            width: 512,
            height: 512,
            seed: 3,
            ..Default::default()
        };
        let path = dir.join("job.trwld");
        let micro_flags = opts.micro_flags;
        let job = WorldGenJob::spawn(
            opts,
            path.clone(),
            WorldLoadFlags::default(),
            false,
            false,
        );
        // Poll until the task lands (the 512² world generates in <1 s).
        let result = loop {
            if let Some(r) = job.take_result() {
                break r;
            }
            std::thread::yield_now();
        };
        assert!(matches!(result, JobResult::Saved(p) if p == path));
        let snap = job.snapshot();
        assert_eq!(snap.total_progress, 1.0);
        // The message is the last pass's label (the world tail's
        // `Lang.gen[87]`, WorldGen.cs:22489) — resolved or the raw key.
        assert!(!snap.message.is_empty());
        // W4-f23 closeout item 1: the debug driver's per-step feed — every
        // REGISTERED step completed, none running after the task lands.
        // `GenPassResults` gets one row per applied pass (`WorldGenerator.cs:
        // 507`); a gate-closed pass is never `AddGenerationPass`ed and appends
        // nothing (`:488-495`). A classic world never registers the Dual
        // Dungeons Dither Snake (`WorldGen.cs:14753-14755`), so the pass-list
        // row for it is a row the loop never runs — count the registered rows.
        let registered = tr_core::worldgen::pipeline::executed_steps(micro_flags)
            .iter()
            .filter(|s| s.enabled && s.vanilla != "Dual Dungeons Dither Snake")
            .count();
        assert_eq!(
            snap.completed_steps, registered,
            "every registered step appended a GenPassResult"
        );
        assert!(snap.current_step.is_empty(), "no pass runs after completion");
        // The saved world loads.
        let (world, _) = tr_core::save::load(&path).unwrap();
        assert_eq!(world.width, 512);
        // W4-c43: the worker's Completed arm stamps the created identity
        // (`CreateMetadata:600-604` + the WorldId roll's stand-in).
        assert_eq!(world.generator_version, tr_core::wld::WORLD_GENERATOR_VERSION);
        assert_ne!(world.unique_id, [0u8; 16], "the GUID is stamped");
        assert!(world.world_id >= 0, "the WorldId roll is in [0, int.MaxValue)");
        assert!(world.creation_time > tr_core::clock::TICKS_AT_UNIX_EPOCH);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The abort arm: `QueuedAbort` set before the worker's first head poll
    /// breaks the loop immediately — no save, `JobResult::Aborted`
    /// (`WorldGen.cs:6595-6598` skips SaveNewWorld).
    #[test]
    fn s1_e_17_the_abort_arm_skips_the_save() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let dir = std::env::temp_dir().join(format!("tr-s1-e-17-abort-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let opts = tr_core::worldgen::GenOptions {
            name: "abort".into(),
            width: 512,
            height: 512,
            seed: 3,
            ..Default::default()
        };
        let path = dir.join("abort.trwld");
        let job = WorldGenJob::spawn_with_abort(
            opts,
            path.clone(),
            WorldLoadFlags::default(),
            false,
            false,
            true,
        );
        assert!(job.queued_abort());
        let result = loop {
            if let Some(r) = job.take_result() {
                break r;
            }
            std::thread::yield_now();
        };
        assert!(matches!(result, JobResult::Aborted));
        assert!(!path.exists(), "the abort saves nothing");
        // The progress never left zero — no pass ran.
        assert_eq!(job.snapshot().total_progress, 0.0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
