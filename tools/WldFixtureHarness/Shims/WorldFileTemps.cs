// S1-J-3 golden-fixture harness — the WorldFile statics + the _temp* snapshot
// fields SaveWorldFlags writes, plus SetTempToOngoing's source statics.
//
// The verbatim SetTempToOngoing (Generated/WorldFileSave.cs) copies the live
// Main/event statics into these temps before the write; the fixture calls it
// exactly as WorldFile.SaveWorld does (WorldFile.cs:870-906). The temps are
// declared here at the C#'s types (WorldFile.cs:96-148).

using System;
using System.Collections.Generic;

namespace Terraria.IO
{
    public static partial class WorldFile
    {
        internal static readonly object IOLock = new object();
        private static double _tempTime;
        private static bool _tempRaining;
        private static float _tempMaxRain;
        private static int _tempRainTime;
        private static bool _tempDayTime;
        private static bool _tempBloodMoon;
        private static bool _tempEclipse;
        private static int _tempMoonPhase;
        private static int _tempCultistDelay;
        private static int _versionNumber;
        private static bool _tempPartyGenuine;
        private static bool _tempPartyManual;
        private static int _tempPartyCooldown;
        private static readonly List<int> TempPartyCelebratingNPCs = new List<int>();
        private static bool _tempSandstormHappening;
        private static int _tempSandstormTimeLeft;
        private static float _tempSandstormSeverity;
        private static float _tempSandstormIntendedSeverity;
        private static bool _tempLanternNightGenuine;
        private static bool _tempLanternNightManual;
        private static bool _tempLanternNightNextNightIsGenuine;
        private static int _tempLanternNightCooldown;
        private static int _tempCoinRain;
        private static int _tempMeteorShowerCount;

        // The static ctor (WorldFile.cs:3955-3965) reads Main.time/dayTime/etc
        // and CultistRitual.delay into the temps; the fixture re-runs
        // SetTempToOngoing after populating the world, so the initial values
        // here only need to be the C# field defaults (zero/false).
    }
}

// SetTempToOngoing's source statics (the live event state). The fixture sets
// the ones it wants persisted; the rest keep the C# defaults.
namespace Terraria.GameContent.Events
{
    public static class CultistRitual
    {
        public static int delay;
    }

    public static class BirthdayParty
    {
        public static bool ManualParty;
        public static bool GenuineParty;
        public static int PartyDaysOnCooldown;
        public static List<int> CelebratingNPCs = new List<int>();
    }

    public static class Sandstorm
    {
        public static bool Happening;
        public static int TimeLeft;
        public static float Severity;
        public static float IntendedSeverity;
    }

    public static class LanternNight
    {
        public static int LanternNightsOnCooldown;
        public static bool GenuineLanterns;
        public static bool ManualLanterns;
        public static bool NextNightIsLanternNight;
    }

    // DD2Event.Save's three bools (Generated/TrackerSave.cs).
    public static partial class DD2Event
    {
        public static bool DownedInvasionT1;
        public static bool DownedInvasionT2;
        public static bool DownedInvasionT3;
    }
}

namespace Terraria
{
    public static partial class WorldGen
    {
        public static int meteorShowerCount;
    }

    public static partial class Main
    {
        public static int coinRain;
    }
}
