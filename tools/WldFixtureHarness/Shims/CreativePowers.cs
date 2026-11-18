// S1-J-3 golden-fixture harness — the creative-power hierarchy. The manager's
// Register/Initialize/SaveToWorld/SaveToPlayer are extracted verbatim
// (Generated/CreativePowerManagerSave.cs); each power's Save is extracted
// verbatim (Generated/CreativePowersSave.cs). This file declares the base-class
// fields those bodies read and the interface surface Register requires.
//
// In the decompile every power is nested inside `public class CreativePowers`
// (CreativePowers.cs:17); the harness keeps that nesting so the generated
// `CreativePowers.FreezeTime` references resolve. The non-save members of
// ICreativePower / the abstract bases are stubbed (throw) — they are never
// called on the save path; only the field layout and the Save bodies matter.

using System;
using System.Collections.Generic;
using System.IO;

namespace Terraria.UI
{
    public class UIElement { }
}

namespace Terraria.GameContent.Creative
{
    public enum PowerPermissionLevel
    {
        LockedForEveryone,
        CanBeChangedByHostAlone,
        CanBeChangedByEveryone
    }

    public class CreativePowerUIElementRequestInfo { }

    public interface ICreativePower
    {
        ushort PowerId { get; set; }
        string ServerConfigName { get; set; }
        PowerPermissionLevel CurrentPermissionLevel { get; set; }
        PowerPermissionLevel DefaultPermissionLevel { get; set; }
        void DeserializeNetMessage(BinaryReader reader, int userId);
        void ProvidePowerButtons(CreativePowerUIElementRequestInfo info, List<Terraria.UI.UIElement> elements);
        bool GetIsUnlocked();
    }

    public interface IPersistentPerWorldContent
    {
        void Save(BinaryWriter writer);
        void Load(BinaryReader reader, int gameVersionSaveWasMadeOn);
        void ValidateWorld(BinaryReader reader, int gameVersionSaveWasMadeOn);
        void Reset();
    }

    public interface IPersistentPerPlayerContent
    {
        void Save(Player player, BinaryWriter writer);
        void Load(Player player, BinaryReader reader, int gameVersionSaveWasMadeOn);
        void ApplyLoadedDataToOutOfPlayerFields(Player player);
        void ResetDataForNewPlayer(Player player);
        void Reset();
    }

    // --- the manager (partial; the save half is generated) -------------------
    public partial class CreativePowerManager
    {
        private class PowerTypeStorage<T> where T : ICreativePower
        {
            public static ushort Id;
            public static string Name;
            public static T Power;
        }

        public static readonly CreativePowerManager Instance = new CreativePowerManager();
        private Dictionary<ushort, ICreativePower> _powersById = new Dictionary<ushort, ICreativePower>();
        private Dictionary<string, ICreativePower> _powersByName = new Dictionary<string, ICreativePower>();
        private ushort _powersCount;
        private static bool _initialized;

        private CreativePowerManager() { }

        public T GetPower<T>() where T : ICreativePower => PowerTypeStorage<T>.Power;
    }

    // --- the wrapper + abstract bases + concrete powers ----------------------
    public partial class CreativePowers
    {
        public abstract class ASharedTogglePower : ICreativePower
        {
            public bool Enabled { get; private set; }
            public void SetPowerInfo(bool enabled) => Enabled = enabled;
            public ushort PowerId { get; set; }
            public string ServerConfigName { get; set; }
            public PowerPermissionLevel CurrentPermissionLevel { get; set; }
            public PowerPermissionLevel DefaultPermissionLevel { get; set; }
            public void DeserializeNetMessage(BinaryReader reader, int userId) => throw new NotImplementedException();
            public void ProvidePowerButtons(CreativePowerUIElementRequestInfo info, List<Terraria.UI.UIElement> elements) => throw new NotImplementedException();
            public abstract bool GetIsUnlocked();
        }

        public abstract class ASharedSliderPower : ICreativePower
        {
            internal float _sliderCurrentValueCache;
            public ushort PowerId { get; set; }
            public string ServerConfigName { get; set; }
            public PowerPermissionLevel CurrentPermissionLevel { get; set; }
            public PowerPermissionLevel DefaultPermissionLevel { get; set; }
            public void DeserializeNetMessage(BinaryReader reader, int userId) => throw new NotImplementedException();
            public void ProvidePowerButtons(CreativePowerUIElementRequestInfo info, List<Terraria.UI.UIElement> elements) => throw new NotImplementedException();
            public abstract bool GetIsUnlocked();
        }

        public abstract class ASharedButtonPower : ICreativePower
        {
            public ushort PowerId { get; set; }
            public string ServerConfigName { get; set; }
            public PowerPermissionLevel CurrentPermissionLevel { get; set; }
            public PowerPermissionLevel DefaultPermissionLevel { get; set; }
            public void DeserializeNetMessage(BinaryReader reader, int userId) => throw new NotImplementedException();
            public void ProvidePowerButtons(CreativePowerUIElementRequestInfo info, List<Terraria.UI.UIElement> elements) => throw new NotImplementedException();
            public abstract bool GetIsUnlocked();
        }

        public abstract class APerPlayerTogglePower : ICreativePower
        {
            internal bool _defaultToggleState;
            private bool[] _perPlayerIsEnabled = new bool[255];
            public bool IsEnabledForPlayer(int playerIndex)
                => playerIndex >= 0 && playerIndex < _perPlayerIsEnabled.Length && _perPlayerIsEnabled[playerIndex];
            public void SetEnabledState(int playerIndex, bool state) => _perPlayerIsEnabled[playerIndex] = state;
            public ushort PowerId { get; set; }
            public string ServerConfigName { get; set; }
            public PowerPermissionLevel CurrentPermissionLevel { get; set; }
            public PowerPermissionLevel DefaultPermissionLevel { get; set; }
            public void DeserializeNetMessage(BinaryReader reader, int userId) => throw new NotImplementedException();
            public void ProvidePowerButtons(CreativePowerUIElementRequestInfo info, List<Terraria.UI.UIElement> elements) => throw new NotImplementedException();
            public abstract bool GetIsUnlocked();
        }

        public abstract class APerPlayerSliderPower : ICreativePower
        {
            internal float _sliderCurrentValueCache;
            internal float _sliderDefaultValue;
            public ushort PowerId { get; set; }
            public string ServerConfigName { get; set; }
            public PowerPermissionLevel CurrentPermissionLevel { get; set; }
            public PowerPermissionLevel DefaultPermissionLevel { get; set; }
            public void DeserializeNetMessage(BinaryReader reader, int userId) => throw new NotImplementedException();
            public void ProvidePowerButtons(CreativePowerUIElementRequestInfo info, List<Terraria.UI.UIElement> elements) => throw new NotImplementedException();
            public abstract bool GetIsUnlocked();
        }

        public partial class FreezeTime : ASharedTogglePower, IPersistentPerWorldContent
        {
            public override bool GetIsUnlocked() => true;
            public void Load(BinaryReader r, int v) => SetPowerInfo(r.ReadBoolean());
            public void ValidateWorld(BinaryReader r, int v) => r.ReadBoolean();
            public void Reset() => SetPowerInfo(false);
        }
        public partial class FreezeRainPower : ASharedTogglePower, IPersistentPerWorldContent
        {
            public override bool GetIsUnlocked() => true;
            public void Load(BinaryReader r, int v) => SetPowerInfo(r.ReadBoolean());
            public void ValidateWorld(BinaryReader r, int v) => r.ReadBoolean();
            public void Reset() => SetPowerInfo(false);
        }
        public partial class FreezeWindDirectionAndStrength : ASharedTogglePower, IPersistentPerWorldContent
        {
            public override bool GetIsUnlocked() => true;
            public void Load(BinaryReader r, int v) => SetPowerInfo(r.ReadBoolean());
            public void ValidateWorld(BinaryReader r, int v) => r.ReadBoolean();
            public void Reset() => SetPowerInfo(false);
        }
        public partial class StopBiomeSpreadPower : ASharedTogglePower, IPersistentPerWorldContent
        {
            public override bool GetIsUnlocked() => true;
            public void Load(BinaryReader r, int v) => SetPowerInfo(r.ReadBoolean());
            public void ValidateWorld(BinaryReader r, int v) => r.ReadBoolean();
            public void Reset() => SetPowerInfo(false);
        }
        public partial class ModifyTimeRate : ASharedSliderPower, IPersistentPerWorldContent
        {
            public override bool GetIsUnlocked() => true;
            public void Load(BinaryReader r, int v) => _sliderCurrentValueCache = r.ReadSingle();
            public void ValidateWorld(BinaryReader r, int v) => r.ReadSingle();
            public void Reset() => _sliderCurrentValueCache = 0f;
        }
        public partial class DifficultySliderPower : ASharedSliderPower, IPersistentPerWorldContent
        {
            public override bool GetIsUnlocked() => true;
            public void Load(BinaryReader r, int v) => _sliderCurrentValueCache = r.ReadSingle();
            public void ValidateWorld(BinaryReader r, int v) => r.ReadSingle();
            public void Reset() => _sliderCurrentValueCache = 0f;
        }
        public partial class GodmodePower : APerPlayerTogglePower, IPersistentPerPlayerContent
        {
            public override bool GetIsUnlocked() => true;
            public void Load(Player p, BinaryReader r, int v) { }
            public void ApplyLoadedDataToOutOfPlayerFields(Player p) { }
            public void ResetDataForNewPlayer(Player p) { }
            public void Reset() { }
        }
        public partial class FarPlacementRangePower : APerPlayerTogglePower, IPersistentPerPlayerContent
        {
            public FarPlacementRangePower() { _defaultToggleState = true; }
            public override bool GetIsUnlocked() => true;
            public void Load(Player p, BinaryReader r, int v) { }
            public void ApplyLoadedDataToOutOfPlayerFields(Player p) { }
            public void ResetDataForNewPlayer(Player p) { }
            public void Reset() { }
        }
        public partial class SpawnRateSliderPerPlayerPower : APerPlayerSliderPower, IPersistentPerPlayerContent
        {
            public override bool GetIsUnlocked() => true;
            public void Load(Player p, BinaryReader r, int v) { }
            public void ApplyLoadedDataToOutOfPlayerFields(Player p) { }
            public void ResetDataForNewPlayer(Player p) { }
            public void Reset() { }
        }

        public class StartDayImmediately : ASharedButtonPower { public override bool GetIsUnlocked() => true; }
        public class StartNoonImmediately : ASharedButtonPower { public override bool GetIsUnlocked() => true; }
        public class StartNightImmediately : ASharedButtonPower { public override bool GetIsUnlocked() => true; }
        public class StartMidnightImmediately : ASharedButtonPower { public override bool GetIsUnlocked() => true; }
        public class ModifyWindDirectionAndStrength : ASharedSliderPower { public override bool GetIsUnlocked() => true; }
        public class ModifyRainPower : ASharedSliderPower { public override bool GetIsUnlocked() => true; }
    }
}
