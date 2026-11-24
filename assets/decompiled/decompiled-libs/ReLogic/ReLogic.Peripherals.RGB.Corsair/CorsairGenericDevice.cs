using System;
using Microsoft.Xna.Framework;

namespace ReLogic.Peripherals.RGB.Corsair;

internal class CorsairGenericDevice : RgbDevice
{
	private readonly CorsairLedColor[] _ledColors;

	protected CorsairGenericDevice(RgbDeviceType deviceType, Fragment fragment, CorsairLedPosition[] ledPositions, DeviceColorProfile colorProfile)
		: base(RgbDeviceVendor.Corsair, deviceType, fragment, colorProfile)
	{
		_ledColors = new CorsairLedColor[LedCount];
		for (int i = 0; i < ledPositions.Length; i++)
		{
			_ledColors[i].LedId = ledPositions[i].LedId;
		}
	}

	public override void Present()
	{
		for (int i = 0; i < LedCount; i++)
		{
			Vector4 processedLedColor = GetProcessedLedColor(i);
			_ledColors[i].R = (int)(processedLedColor.X * 255f);
			_ledColors[i].G = (int)(processedLedColor.Y * 255f);
			_ledColors[i].B = (int)(processedLedColor.Z * 255f);
		}
		if (_ledColors.Length != 0)
		{
			NativeMethods.CorsairSetLedsColorsAsync(_ledColors.Length, _ledColors, IntPtr.Zero, IntPtr.Zero);
		}
	}
}
