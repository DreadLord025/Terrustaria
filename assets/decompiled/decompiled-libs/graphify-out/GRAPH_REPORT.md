# Graph Report - decompiled-libs  (2026-10-30)

## Corpus Check
- Corpus is ~31,146 words - fits in a single context window. You may not need a graph.

## Summary
- 1992 nodes · 3659 edges · 102 communities (76 shown, 26 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 300 edges (avg confidence: 0.82)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- ReLogic.Peripherals.RGB.Corsair: CorsairLedId
- ReLogic.Peripherals.RGB.Razer: RazerKey
- ReLogic.Peripherals.RGB.Logitech: KeyName
- ReLogic.Content: AssetRepository
- ReLogic.Utilities: Vector2D
- ReLogic.Peripherals.RGB.Corsair: system
- ReLogic.Content: ReLogic.Content
- ReLogic.Utilities: SlotId
- ReLogic.Peripherals.RGB.Logitech: DllImport
- ReLogic.OS.Base: GameWindow
- ReLogic.Content.Readers: Stream
- ReLogic.Peripherals.RGB.Razer: microsoft_xna_framework
- ReLogic.Content: AsyncAssetLoader
- ReLogic.Peripherals.RGB.SteelSeries: GameSenseConnection
- ReLogic.Content: FileChangeWatcher
- ReLogic.Localization.IME.Windows: NativeMethods
- ReLogic.Peripherals.RGB: RgbKey
- ReLogic.OS: Platform
- ReLogic.OS.Base: PathService
- ReLogic.Content: IContentSource
- ReLogic.Content.Sources: XnaContentSource
- ReLogic.OS.Windows: NativeMethods
- ReLogic.Peripherals.RGB: RgbDevice
- ReLogic.Peripherals.RGB: ChromaEngine
- ReLogic.OS.Base: Clipboard
- ReLogic.OS.Windows: IntPtr
- ReLogic.Localization.IME: PlatformIme
- ReLogic.Peripherals.RGB: Fragment
- ReLogic.Content: FileSystemContentSource
- ReLogic.Peripherals.RGB.Corsair: CorsairLogicalLayout
- ReLogic.Peripherals.RGB.Razer: EffectHandle
- ReLogic.Peripherals.RGB.Corsair: NativeMethods
- ReLogic.Peripherals.RGB.Corsair: DeviceColorProfile
- ReLogic.Text: StringReaderWrapExtension
- ReLogic.Peripherals.RGB.Razer: NativeMethods
- ReLogic.Graphics: Color
- root: AssemblyInfo.cs
- ReLogic.Peripherals.RGB.SteelSeries: SteelSeriesDeviceGroup
- ReLogic.Graphics: BasicDebugDrawer
- ReLogic.OS.Windows: ConsoleMode
- ReLogic.Peripherals.RGB.Razer: RzResult
- ReLogic.OS.Linux: ReLogic.OS.Base
- ReLogic.Peripherals.RGB.SteelSeries: SteelSeriesKeyboard
- ReLogic.Content: Asset
- ReLogic.Localization.IME: UnsupportedPlatformIme
- ReLogic.Peripherals.RGB: ARgbGameValueTracker
- ReLogic.Peripherals.RGB.SteelSeries: system_collections_generic
- ReLogic.Content.Sources: ZipContentSource
- ReLogic.Utilities: MultiTimer
- ReLogic.Peripherals.RGB: EffectDetailLevel
- ReLogic.Peripherals.RGB: ChromaShader
- ReLogic.Peripherals.RGB: ShaderLayer
- ReLogic.OS.Windows: Action
- ReLogic.Graphics: DynamicSpriteFont
- ReLogic.OS.Windows: WindowsMessageHook
- ReLogic.Text: WrappedTextBuilder
- ReLogic.Content: IAssetRepository
- ReLogic.Reflection: IdDictionary
- ReLogic.Peripherals.RGB: ShaderSelector
- ReLogic.Utilities: ReLogic.Utilities
- ReLogic.Localization.IME: WindowsIme
- ReLogic.Localization.IME: FnaIme
- ReLogic.OS.Windows: WindowsMouseNotifier
- ReLogic.Peripherals.RGB: ChromaCondition
- ReLogic.Peripherals.RGB.Corsair: CorsairPhysicalLayout
- ReLogic.Peripherals.RGB.Razer: KeyboardEffectType
- ReLogic.Peripherals.RGB.Razer: MouseEffectType
- ReLogic.Peripherals.RGB: ReLogic.Peripherals.RGB
- ReLogic.Text: DynamicSpriteFont.cs
- ReLogic.Peripherals.RGB.Razer: .Initialize()
- ReLogic.Graphics: .Read()
- ReLogic.Threading: FastParallel
- ReLogic.Threading: AsyncAssetLoader.cs
- ReLogic.Peripherals.RGB.Razer: KeypadEffectType
- ReLogic.Peripherals.RGB: RgbDeviceType
- ReLogic.Peripherals.RGB: VendorColorProfile
- ReLogic.Peripherals.RGB.Razer: RazerKeyboard
- ReLogic.Peripherals.RGB.Razer: .CreateMouseEffect()
- ReLogic.Peripherals.RGB.Corsair: CorsairError
- ReLogic.Peripherals.RGB.Razer: MousepadEffectType
- ReLogic.Peripherals.RGB: RgbDeviceVendor
- ReLogic.OS.Windows: FlashFlags
- ReLogic.Peripherals.RGB.Corsair: CorsairDeviceType
- ReLogic.Peripherals.RGB.Razer: HeadsetEffectType
- ReLogic.Text: GlyphMetrics
- ReLogic.Peripherals.RGB: ChromaPipeline
- ReLogic.Peripherals.RGB: CorsairKeyboard
- ReLogic.Peripherals.RGB: RgbKeyEffect
- ReLogic.OS.Windows: FileOperationType
- ReLogic.Peripherals.RGB: BlendMode
- ReLogic.Peripherals.RGB.Corsair: CorsairDeviceInfo
- ReLogic.Peripherals.RGB.Razer: ChromaLinkEffectType
- ReLogic.Text: StringBuilderExtension

## God Nodes (most connected - your core abstractions)
1. `CorsairLedId` - 196 edges
2. `RazerKey` - 126 edges
3. `KeyName` - 113 edges
4. `Vector2D` - 63 edges
5. `AssetRepository` - 49 edges
6. `RgbDevice` - 46 edges
7. `GameSenseConnection` - 39 edges
8. `IContentSource` - 35 edges
9. `RgbKey` - 34 edges
10. `DynamicSpriteFont` - 29 edges

## Surprising Connections (you probably didn't know these)
- `LinuxPlatform` --calls--> `Clipboard`  [INFERRED]
  ReLogic/ReLogic.OS.Linux/LinuxPlatform.cs → ReLogic/ReLogic.OS.Linux/Clipboard.cs
- `LinuxPlatform` --calls--> `PathService`  [INFERRED]
  ReLogic/ReLogic.OS.Linux/LinuxPlatform.cs → ReLogic/ReLogic.OS.Linux/PathService.cs
- `LinuxPlatform` --calls--> `WindowService`  [INFERRED]
  ReLogic/ReLogic.OS.Linux/LinuxPlatform.cs → ReLogic/ReLogic.OS.Linux/WindowService.cs
- `OsxPlatform` --calls--> `Clipboard`  [INFERRED]
  ReLogic/ReLogic.OS.OSX/OsxPlatform.cs → ReLogic/ReLogic.OS.OSX/Clipboard.cs
- `OsxPlatform` --calls--> `PathService`  [INFERRED]
  ReLogic/ReLogic.OS.OSX/OsxPlatform.cs → ReLogic/ReLogic.OS.OSX/PathService.cs

## Import Cycles
- None detected.

## Communities (102 total, 26 thin omitted)

### Community 0 - "ReLogic.Peripherals.RGB.Corsair: CorsairLedId"
Cohesion: 0.01
Nodes (193): CorsairLedId, CLH_LeftLogo, CLH_RightLogo, CLI_Invalid, CLI_Last, CLK_0, CLK_1, CLK_2 (+185 more)

### Community 1 - "ReLogic.Peripherals.RGB.Razer: RazerKey"
Cohesion: 0.02
Nodes (125): RazerKey, A, B, Backspace, C, CapsLock, D, D0 (+117 more)

### Community 2 - "ReLogic.Peripherals.RGB.Logitech: KeyName"
Cohesion: 0.02
Nodes (105): KeyName, A, APOSTROPHE, APPLICATION_SELECT, ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP (+97 more)

### Community 3 - "ReLogic.Content: AssetRepository"
Cohesion: 0.08
Nodes (20): AssetChangeWatcher, AssetRepository, AssetLoadFailHandler, AssetWatcherUpdateFailedHandler, AssetWatcherValueUpdatedHandler, ContentFileUpdatedHandler, IsAsyncLoadingEnabled, LoadedAssets (+12 more)

### Community 4 - "ReLogic.Utilities: Vector2D"
Cohesion: 0.04
Nodes (6): Vector2D, DebugDisplayString, One, UnitX, UnitY, Zero

### Community 5 - "ReLogic.Peripherals.RGB.Corsair: system"
Cohesion: 0.10
Nodes (9): ReLogic.OS.Windows, ReLogic.Localization.IME.Windows, ReLogic.Localization.IME, ReLogic.Peripherals.RGB.Corsair, Imm, WindowsMessage, CorsairGenericDevice, CorsairHeadset (+1 more)

### Community 6 - "ReLogic.Content: ReLogic.Content"
Cohesion: 0.09
Nodes (7): ReLogic.Content, ReLogic.IO, ReLogic.Content.Sources, ReLogic.Content.Readers, AssetWatcherUpdateFailed, ContentFileUpdated, FailedToLoadAssetCustomAction

### Community 7 - "ReLogic.Utilities: SlotId"
Cohesion: 0.09
Nodes (14): IntFloat, ReinterpretCast, UIntFloat, SlotId, Index, IsActive, IsValid, Key (+6 more)

### Community 8 - "ReLogic.Peripherals.RGB.Logitech: DllImport"
Cohesion: 0.12
Nodes (3): LogitechKeyboard, LogitechSingleLightDevice, NativeMethods

### Community 9 - "ReLogic.OS.Base: GameWindow"
Cohesion: 0.14
Nodes (3): SDLWindowService, IWindowService, WindowService

### Community 10 - "ReLogic.Content.Readers: Stream"
Cohesion: 0.10
Nodes (6): AssetLoadException, AssetReaderCollection, IAssetReader, PngReader, InternalContentManager, XnbReader

### Community 11 - "ReLogic.Peripherals.RGB.Razer: microsoft_xna_framework"
Cohesion: 0.10
Nodes (4): ReLogic.Peripherals.RGB.Logitech, ReLogic.Peripherals.RGB.Razer, ReLogic.Graphics, RazerHelper

### Community 12 - "ReLogic.Content: AsyncAssetLoader"
Cohesion: 0.09
Nodes (11): AsyncAssetLoader, AssetsReadyForTransfer, IsRunning, Remaining, IAsyncAssetLoader, IsRunning, Remaining, AsyncActionDispatcher (+3 more)

### Community 14 - "ReLogic.Content: FileChangeWatcher"
Cohesion: 0.11
Nodes (5): FileChangeWatcher, FileUpdate, ConsoleOutputMirror, DoubleWriter, Encoding

### Community 15 - "ReLogic.Localization.IME.Windows: NativeMethods"
Cohesion: 0.08
Nodes (6): HARDWAREINPUT, INPUT, INPUTUNION, KEYBDINPUT, MOUSEINPUT, NativeMethods

### Community 16 - "ReLogic.Peripherals.RGB: RgbKey"
Cohesion: 0.11
Nodes (8): HotkeyCollection, RgbKey, BaseColor, CurrentColor, CurrentIntegerRepresentation, Effect, IsVisible, TargetColor

### Community 17 - "ReLogic.OS: Platform"
Cohesion: 0.10
Nodes (9): Platform, IsLinux, IsOSX, IsWindows, PlatformType, Linux, OSX, Windows (+1 more)

### Community 18 - "ReLogic.OS.Base: PathService"
Cohesion: 0.10
Nodes (5): PathService, IPathService, PathService, PathService, PathService

### Community 19 - "ReLogic.Content: IContentSource"
Cohesion: 0.16
Nodes (6): AssetLoader, IAssetLoader, LoadAssetDelegate, IContentSource, ContentValidator, FileWatcherPath

### Community 20 - "ReLogic.Content.Sources: XnaContentSource"
Cohesion: 0.11
Nodes (7): AssetPathHelper, XnaContentSource, ContentValidator, FileWatcherPath, XnaDirectContentSource, ContentValidator, FileWatcherPath

### Community 21 - "ReLogic.OS.Windows: NativeMethods"
Cohesion: 0.12
Nodes (11): DeviceCap, DesktopVertRes, VertRes, NativeMethods, RAWINPUTDEVICELIST, SHFILEOPSTRUCT, StdHandleType, Error (+3 more)

### Community 22 - "ReLogic.Peripherals.RGB: RgbDevice"
Cohesion: 0.12
Nodes (12): CorsairDeviceGroup, State, AnyDevicesAdded, DeviceControlGranted, HandshakeCompleted, Unitialized, LogitechDeviceGroup, RazerDeviceGroup (+4 more)

### Community 23 - "ReLogic.Peripherals.RGB: ChromaEngine"
Cohesion: 0.15
Nodes (4): ChromaEngine, FrameTimeInSeconds, RgbDeviceGroup, IsEnabled

### Community 24 - "ReLogic.OS.Base: Clipboard"
Cohesion: 0.11
Nodes (8): Clipboard, MultiLineValue, Value, IClipboard, MultiLineValue, Value, Clipboard, Clipboard

### Community 25 - "ReLogic.OS.Windows: IntPtr"
Cohesion: 0.13
Nodes (3): NativeMethods, FlashInfo, CorsairLedPositions

### Community 26 - "ReLogic.Localization.IME: PlatformIme"
Cohesion: 0.12
Nodes (12): IImeService, CandidateCount, CompositionString, IsCandidateListVisible, IsEnabled, SelectedCandidate, PlatformIme, CandidateCount (+4 more)

### Community 27 - "ReLogic.Peripherals.RGB: Fragment"
Cohesion: 0.13
Nodes (5): Fragment, CanvasBottomRight, CanvasCenter, CanvasSize, CanvasTopLeft

### Community 28 - "ReLogic.Content: FileSystemContentSource"
Cohesion: 0.17
Nodes (8): ContentRejectionFromFailedLoad_ByAssetExtensionType, IContentValidator, IRejectionReason, RejectedAssetCollection, FileSystemContentSource, ContentValidator, FileCount, FileWatcherPath

### Community 29 - "ReLogic.Peripherals.RGB.Corsair: CorsairLogicalLayout"
Cohesion: 0.10
Nodes (20): CorsairLogicalLayout, CLL_BE, CLL_BR, CLL_CH, CLL_CN, CLL_DE, CLL_ES, CLL_EU (+12 more)

### Community 30 - "ReLogic.Peripherals.RGB.Razer: EffectHandle"
Cohesion: 0.24
Nodes (4): EffectHandle, RazerKeypad, RazerLink, RazerMousepad

### Community 31 - "ReLogic.Peripherals.RGB.Corsair: NativeMethods"
Cohesion: 0.14
Nodes (5): CorsairAccessMode, CAM_ExclusiveLightingControl, CorsairLedColor, CorsairProtocolDetails, NativeMethods

### Community 32 - "ReLogic.Peripherals.RGB.Corsair: DeviceColorProfile"
Cohesion: 0.15
Nodes (7): CorsairHelper, CorsairLedPosition, CorsairMouse, DeviceColorProfile, BlueMultiplier, GreenMultiplier, RedMultiplier

### Community 33 - "ReLogic.Text: StringReaderWrapExtension"
Cohesion: 0.16
Nodes (7): CharacterSet, StringReaderWrapExtension, WrapScanMode, NewLine, None, Space, Word

### Community 34 - "ReLogic.Peripherals.RGB.Razer: NativeMethods"
Cohesion: 0.14
Nodes (4): CustomChromaLinkEffect, CustomKeypadEffect, CustomMousepadEffect, NativeMethods

### Community 36 - "root: AssemblyInfo.cs"
Cohesion: 0.11
Nodes (3): ReLogic.Reflection, AttributeUtilities, TypeAttributeCache

### Community 37 - "ReLogic.Peripherals.RGB.SteelSeries: SteelSeriesDeviceGroup"
Cohesion: 0.14
Nodes (4): ChromaEngineUpdateEventArgs, ClientStateEvent, EventNames, SteelSeriesDeviceGroup

### Community 39 - "ReLogic.OS.Windows: ConsoleMode"
Cohesion: 0.10
Nodes (18): ConsoleMode, AutoPosition, EchoInput, ExtendedFlags, InsertMode, LineInput, MouseInput, ProcessedInput (+10 more)

### Community 40 - "ReLogic.Peripherals.RGB.Razer: RzResult"
Cohesion: 0.11
Nodes (18): RzResult, AccessDenied, AlreadyInitialized, DeviceNotAvailable, DeviceNotConnected, Failed, Invalid, InvalidHandle (+10 more)

### Community 41 - "ReLogic.OS.Linux: ReLogic.OS.Base"
Cohesion: 0.17
Nodes (5): ReLogic.OS.Linux, ReLogic.OS.OSX, ReLogic.OS.Base, Clipboard, WindowService

### Community 42 - "ReLogic.Peripherals.RGB.SteelSeries: SteelSeriesKeyboard"
Cohesion: 0.26
Nodes (6): ColorKey, IGameSenseDevice, IGameSenseUpdater, SteelSeriesEventRelay, SteelSeriesKeyboard, SteelSeriesSecondaryDeviceByZone

### Community 43 - "ReLogic.Content: Asset"
Cohesion: 0.13
Nodes (12): Asset, IsDisposed, IsLoaded, Name, Source, State, Value, AssetValueUpdatedHandler (+4 more)

### Community 44 - "ReLogic.Localization.IME: UnsupportedPlatformIme"
Cohesion: 0.15
Nodes (9): UnsupportedPlatformIme, CandidateCount, CompositionString, IsCandidateListVisible, SelectedCandidate, LinuxPlatform, OsxPlatform, WindowService (+1 more)

### Community 45 - "ReLogic.Peripherals.RGB: ARgbGameValueTracker"
Cohesion: 0.17
Nodes (4): ARgbGameValueTracker, ARgbGameValueTracker, IntRgbGameValueTracker, GameSenseSpecificInfo

### Community 47 - "ReLogic.Content.Sources: ZipContentSource"
Cohesion: 0.16
Nodes (4): ZipContentSource, ContentValidator, EntryCount, FileWatcherPath

### Community 48 - "ReLogic.Utilities: MultiTimer"
Cohesion: 0.13
Nodes (4): Crc32, MultiTimer, TimerData, Average

### Community 49 - "ReLogic.Peripherals.RGB: EffectDetailLevel"
Cohesion: 0.20
Nodes (6): EffectDetailLevel, High, Low, Max, ShaderBlendState, ShaderOperation

### Community 50 - "ReLogic.Peripherals.RGB: ChromaShader"
Cohesion: 0.20
Nodes (5): BoundProcessor, ChromaShader, Processor, RgbProcessorAttribute, IsTransparent

### Community 51 - "ReLogic.Peripherals.RGB: ShaderLayer"
Cohesion: 0.14
Nodes (13): ShaderLayer, Alert, Background, Biome, BiomeModifier, Boss, Bottom, Count (+5 more)

### Community 53 - "ReLogic.Graphics: DynamicSpriteFont"
Cohesion: 0.23
Nodes (4): DrawCharacter, DynamicSpriteFont, CharacterSpacing, LineSpacing

### Community 55 - "ReLogic.Text: WrappedTextBuilder"
Cohesion: 0.28
Nodes (5): IFontMetrics, CharacterSpacing, LineSpacing, NonBreakingText, WrappedTextBuilder

### Community 56 - "ReLogic.Content: IAssetRepository"
Cohesion: 0.15
Nodes (11): AssetValueUpdated, AssetWatcherValueUpdated, IAssetRepository, AssetLoadFailHandler, AssetValueUpdatedHandler, AssetWatcherUpdateFailedHandler, AssetWatcherValueUpdatedHandler, ContentFileUpdatedHandler (+3 more)

### Community 59 - "ReLogic.Utilities: ReLogic.Utilities"
Cohesion: 0.17
Nodes (3): ReLogic.OS, ReLogic.Utilities, XnaExtensions

### Community 60 - "ReLogic.Localization.IME: WindowsIme"
Cohesion: 0.20
Nodes (5): WindowsIme, CandidateCount, CompositionString, IsCandidateListVisible, SelectedCandidate

### Community 62 - "ReLogic.Localization.IME: FnaIme"
Cohesion: 0.17
Nodes (5): FnaIme, CandidateCount, CompositionString, IsCandidateListVisible, SelectedCandidate

### Community 64 - "ReLogic.Peripherals.RGB: ChromaCondition"
Cohesion: 0.23
Nodes (5): ChromaCondition, Custom, ConditionalShader, IsActive, IsVisible

### Community 65 - "ReLogic.Peripherals.RGB.Corsair: CorsairPhysicalLayout"
Cohesion: 0.17
Nodes (11): CorsairPhysicalLayout, CPL_BR, CPL_Invalid, CPL_JP, CPL_KR, CPL_UK, CPL_US, CPL_Zones1 (+3 more)

### Community 66 - "ReLogic.Peripherals.RGB.Razer: KeyboardEffectType"
Cohesion: 0.18
Nodes (11): KeyboardEffectType, Breathing, Custom, CustomKey, Invalid, None, Reactive, Reserved (+3 more)

### Community 67 - "ReLogic.Peripherals.RGB.Razer: MouseEffectType"
Cohesion: 0.18
Nodes (11): MouseEffectType, Blinking, Breathing, Custom, Custom2, Invalid, None, Reactive (+3 more)

### Community 70 - "ReLogic.Peripherals.RGB.Razer: .Initialize()"
Cohesion: 0.22
Nodes (3): DeviceInitializationException, CustomHeadsetEffect, RazerHeadset

### Community 71 - "ReLogic.Graphics: .Read()"
Cohesion: 0.36
Nodes (3): SpriteCharacterData, DynamicSpriteFontReader, FontPage

### Community 72 - "ReLogic.Threading: FastParallel"
Cohesion: 0.33
Nodes (4): FastParallel, ForceTasksOnCallingThread, RangeTask, ParallelForAction

### Community 74 - "ReLogic.Peripherals.RGB.Razer: KeypadEffectType"
Cohesion: 0.22
Nodes (9): KeypadEffectType, Breathing, Custom, Invalid, None, Reactive, Spectrumcycling, Static (+1 more)

### Community 75 - "ReLogic.Peripherals.RGB: RgbDeviceType"
Cohesion: 0.22
Nodes (8): RgbDeviceType, Generic, Headset, Keyboard, Keypad, Mouse, Mousepad, Virtual

### Community 76 - "ReLogic.Peripherals.RGB: VendorColorProfile"
Cohesion: 0.22
Nodes (8): VendorColorProfile, Generic, Headset, Keyboard, Keypad, Mouse, Mousepad, Virtual

### Community 79 - "ReLogic.Peripherals.RGB.Corsair: CorsairError"
Cohesion: 0.25
Nodes (7): CorsairError, CE_IncompatibleProtocol, CE_InvalidArguments, CE_NoControl, CE_ProtocolHandshakeMissing, CE_ServerNotFound, CE_Success

### Community 80 - "ReLogic.Peripherals.RGB.Razer: MousepadEffectType"
Cohesion: 0.25
Nodes (8): MousepadEffectType, Breathing, Custom, Invalid, None, Spectrumcycling, Static, Wave

### Community 81 - "ReLogic.Peripherals.RGB: RgbDeviceVendor"
Cohesion: 0.25
Nodes (7): RgbDeviceVendor, Corsair, Logitech, Razer, SteelSeries, Unknown, Virtual

### Community 82 - "ReLogic.OS.Windows: FlashFlags"
Cohesion: 0.29
Nodes (7): FlashFlags, Caption, CaptionAndTray, Stop, Timer, Tray, UntilFocused

### Community 83 - "ReLogic.Peripherals.RGB.Corsair: CorsairDeviceType"
Cohesion: 0.29
Nodes (6): CorsairDeviceType, CDT_Headset, CDT_Keyboard, CDT_Mouse, CDT_MouseMat, CDT_Unknown

### Community 84 - "ReLogic.Peripherals.RGB.Razer: HeadsetEffectType"
Cohesion: 0.29
Nodes (7): HeadsetEffectType, Breathing, Custom, Invalid, None, Spectrumcycling, Static

### Community 85 - "ReLogic.Text: GlyphMetrics"
Cohesion: 0.40
Nodes (3): GlyphMetrics, KernedWidth, KernedWidthOnNewLine

### Community 88 - "ReLogic.Peripherals.RGB: RgbKeyEffect"
Cohesion: 0.33
Nodes (5): RgbKeyEffect, Clear, Fade, Flashing, Solid

### Community 90 - "ReLogic.OS.Windows: FileOperationType"
Cohesion: 0.40
Nodes (5): FileOperationType, FO_COPY, FO_DELETE, FO_MOVE, FO_RENAME

### Community 91 - "ReLogic.Peripherals.RGB: BlendMode"
Cohesion: 0.40
Nodes (4): BlendMode, GlobalOpacityOnly, None, PerPixelOpacity

### Community 92 - "ReLogic.Peripherals.RGB.Corsair: CorsairDeviceInfo"
Cohesion: 0.40
Nodes (4): CorsairDeviceCaps, CDC_Lighting, CDC_None, CorsairDeviceInfo

### Community 94 - "ReLogic.Peripherals.RGB.Razer: ChromaLinkEffectType"
Cohesion: 0.40
Nodes (5): ChromaLinkEffectType, Custom, Invalid, None, Static

## Knowledge Gaps
- **729 isolated node(s):** `ContentValidator`, `FileCount`, `FileWatcherPath`, `ContentValidator`, `FileWatcherPath` (+724 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 917 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **26 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `CorsairLedId` connect `ReLogic.Peripherals.RGB.Corsair: CorsairLedId` to `ReLogic.Peripherals.RGB.Corsair: DeviceColorProfile`, `ReLogic.Peripherals.RGB.Corsair: NativeMethods`, `ReLogic.Peripherals.RGB.Corsair: system`, `ReLogic.Peripherals.RGB: CorsairKeyboard`?**
  _High betweenness centrality (0.196) - this node is a cross-community bridge._
- **What connects `ContentValidator`, `FileCount`, `FileWatcherPath` to the rest of the system?**
  _729 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `ReLogic.Peripherals.RGB.Corsair: CorsairLedId` be split into smaller, more focused modules?**
  _Cohesion score 0.010362694300518135 - nodes in this community are weakly interconnected._
- **Why does `RazerKeyboard` connect `ReLogic.Peripherals.RGB.Razer: RazerKeyboard` to `ReLogic.Peripherals.RGB.Corsair: DeviceColorProfile`, `ReLogic.Peripherals.RGB.Razer: RazerKey`, `ReLogic.Content: AssetRepository`, `ReLogic.Peripherals.RGB.Razer: .Initialize()`, `ReLogic.Peripherals.RGB.Razer: microsoft_xna_framework`, `ReLogic.Peripherals.RGB: RgbKey`, `ReLogic.Peripherals.RGB: .ProcessLedColor()`, `ReLogic.Peripherals.RGB: CorsairKeyboard`, `ReLogic.Content.Sources: List`, `ReLogic.Peripherals.RGB.Razer: EffectHandle`?**
  _High betweenness centrality (0.148) - this node is a cross-community bridge._
- **Should `ReLogic.Peripherals.RGB.Razer: RazerKey` be split into smaller, more focused modules?**
  _Cohesion score 0.016 - nodes in this community are weakly interconnected._
- **Why does `RazerKey` connect `ReLogic.Peripherals.RGB.Razer: RazerKey` to `ReLogic.Peripherals.RGB.Razer: microsoft_xna_framework`, `ReLogic.Peripherals.RGB.Razer: RazerKeyboard`?**
  _High betweenness centrality (0.138) - this node is a cross-community bridge._
- **Should `ReLogic.Peripherals.RGB.Logitech: KeyName` be split into smaller, more focused modules?**
  _Cohesion score 0.01904761904761905 - nodes in this community are weakly interconnected._