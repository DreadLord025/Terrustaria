using System.Collections.Generic;
using Microsoft.Xna.Framework;
using Newtonsoft.Json.Linq;
using SteelSeries.GameSense;
using SteelSeries.GameSense.DeviceZone;

namespace ReLogic.Peripherals.RGB.SteelSeries;

internal class SteelSeriesSecondaryDeviceByZone : RgbDevice, IGameSenseDevice, IGameSenseUpdater
{
	private string _zoneTarget;

	private int _xPosition;

	private int _yPosition;

	private ColorKey _colorKey;

	private List<JObject> _requestList = new List<JObject>();

	public SteelSeriesSecondaryDeviceByZone(DeviceColorProfile colorProfile, RgbDeviceType type, string zoneNameToCheck, int xPosition, int yPosition)
		: base(RgbDeviceVendor.SteelSeries, type, Fragment.FromGrid(new Rectangle(xPosition, yPosition, 1, 1)), colorProfile)
	{
		_zoneTarget = zoneNameToCheck;
		PreferredLevelOfDetail = EffectDetailLevel.High;
		_xPosition = xPosition;
		_yPosition = yPosition;
	}

	public override void Present()
	{
		Vector4 processedLedColor = GetProcessedLedColor(0);
		Color color = new Color(processedLedColor);
		_colorKey.UpdateColor(color, isVisible: true);
	}

	public List<JObject> TryGetEventUpdateRequest()
	{
		_requestList.Clear();
		JObject val = _colorKey.TryGettingRequest();
		if (val != null)
		{
			_requestList.Add(val);
		}
		return _requestList;
	}

	public void CollectEventsToTrack(Bind_Event[] bindEvents, ARgbGameValueTracker[] miscEvents)
	{
		foreach (Bind_Event val in bindEvents)
		{
			AbstractHandler obj = val.handlers[0];
			ContextColorEventHandlerType val2 = (ContextColorEventHandlerType)(object)((obj is ContextColorEventHandlerType) ? obj : null);
			if (val2 != null)
			{
				AbstractIlluminationDevice_Zone deviceZone = val2.DeviceZone;
				RGBZonedDevice val3 = (RGBZonedDevice)(object)((deviceZone is RGBZonedDevice) ? deviceZone : null);
				if (val3 != null && !(((AbstractIlluminationDevice_StandardZone)val3).zone != _zoneTarget))
				{
					ColorKey colorKey = new ColorKey
					{
						EventName = val.eventName,
						TriggerName = val2.ContextFrameKey
					};
					_colorKey = colorKey;
					break;
				}
			}
		}
	}
}
