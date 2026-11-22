using System.Collections.Generic;
using System.Linq;
using Microsoft.Xna.Framework;
using Newtonsoft.Json.Linq;
using SteelSeries.GameSense;

namespace ReLogic.Peripherals.RGB.SteelSeries;

internal class SteelSeriesKeyboard : RgbKeyboard, IGameSenseDevice, IGameSenseUpdater
{
	public const string EVENT_ID_BITMAP = "DO_RAINBOWS";

	private const int HowManyKeysGameSenseKeyboardUses = 132;

	private const int Rows = 6;

	private const int Columns = 22;

	private readonly int[][] _keyboardKeyColors = new int[132][];

	private readonly Dictionary<string, ColorKey> _keyboardTriggersForLookup = new Dictionary<string, ColorKey>();

	private readonly List<ColorKey> _keyboardTriggersForIteration = new List<ColorKey>();

	private readonly List<ARgbGameValueTracker> eventTrackers = new List<ARgbGameValueTracker>();

	private int _timesSent;

	private List<JObject> _updateRequestsList = new List<JObject>();

	public SteelSeriesKeyboard(DeviceColorProfile colorProfile)
		: base(RgbDeviceVendor.SteelSeries, Fragment.FromGrid(new Rectangle(0, 0, 22, 6)), colorProfile)
	{
		for (int i = 0; i < _keyboardKeyColors.Length; i++)
		{
			_keyboardKeyColors[i] = new int[3];
		}
	}

	public override void Render(IEnumerable<RgbKey> keys)
	{
		foreach (RgbKey key in keys)
		{
			if (_keyboardTriggersForLookup.TryGetValue(key.KeyTriggerName, out var value))
			{
				Color color = ProcessLedColor(key.CurrentColor);
				value.UpdateColor(color, key.IsVisible);
			}
		}
	}

	public override void Present()
	{
		for (int i = 0; i < LedCount; i++)
		{
			Vector4 processedLedColor = GetProcessedLedColor(i);
			int[] obj = _keyboardKeyColors[i];
			obj[0] = (int)(processedLedColor.X * 255f);
			obj[1] = (int)(processedLedColor.Y * 255f);
			obj[2] = (int)(processedLedColor.Z * 255f);
		}
	}

	public List<JObject> TryGetEventUpdateRequest()
	{
		_updateRequestsList.Clear();
		_updateRequestsList.Add(GetBitmapRequestForFullKeyboard());
		for (int i = 0; i < _keyboardTriggersForIteration.Count; i++)
		{
			JObject val = _keyboardTriggersForIteration[i].TryGettingRequest();
			if (val != null)
			{
				_updateRequestsList.Add(val);
			}
		}
		return _updateRequestsList;
	}

	private JObject GetBitmapRequestForFullKeyboard()
	{
		//IL_0000: Unknown result type (might be due to invalid IL or missing references)
		//IL_0006: Expected Obj, but got Unknown
		//IL_00d7: Unknown result type (might be due to invalid IL or missing references)
		//IL_00dd: Expected Obj, but got Unknown
		//IL_011e: Unknown result type (might be due to invalid IL or missing references)
		//IL_0123: Unknown result type (might be due to invalid IL or missing references)
		//IL_0138: Unknown result type (might be due to invalid IL or missing references)
		//IL_0145: Expected Obj, but got Unknown
		JObject val = new JObject();
		val.Add("bitmap", JToken.FromObject((object)_keyboardKeyColors));
		List<string> list = (from x in _keyboardTriggersForIteration
			where x.IsVisible
			select x.EventName).ToList();
		IEnumerable<string> collection = from x in eventTrackers
			where x.IsVisible
			select x.EventName;
		list.AddRange(collection);
		val.Add("excluded-events", JToken.FromObject((object)list));
		JObject val2 = new JObject();
		val2.Add("frame", (JToken)(object)val);
		val2.Add("value", JToken.op_Implicit(_timesSent));
		_timesSent++;
		if (_timesSent >= 100)
		{
			_timesSent = 1;
		}
		JObject val3 = new JObject();
		val3.Add("event", JToken.op_Implicit("DO_RAINBOWS"));
		val3.Add("data", (JToken)(object)val2);
		return val3;
	}

	public void CollectEventsToTrack(Bind_Event[] bindEvents, ARgbGameValueTracker[] miscEvents)
	{
		foreach (Bind_Event val in bindEvents)
		{
			if (val.handlers.Length >= 1)
			{
				AbstractHandler obj = val.handlers[0];
				ContextColorEventHandlerType val2 = (ContextColorEventHandlerType)(object)((obj is ContextColorEventHandlerType) ? obj : null);
				if (val2 != null)
				{
					ColorKey colorKey = new ColorKey
					{
						EventName = val.eventName,
						TriggerName = val2.ContextFrameKey
					};
					_keyboardTriggersForLookup.Add(val2.ContextFrameKey, colorKey);
					_keyboardTriggersForIteration.Add(colorKey);
				}
			}
		}
		eventTrackers.AddRange(miscEvents);
	}
}
