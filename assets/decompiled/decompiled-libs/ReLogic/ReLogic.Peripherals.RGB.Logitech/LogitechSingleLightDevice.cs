using Microsoft.Xna.Framework;

namespace ReLogic.Peripherals.RGB.Logitech;

internal class LogitechSingleLightDevice : RgbDevice
{
	public LogitechSingleLightDevice(DeviceColorProfile colorProfile)
		: base(RgbDeviceVendor.Logitech, RgbDeviceType.Generic, Fragment.FromGrid(new Rectangle(30, 0, 1, 1)), colorProfile)
	{
		PreferredLevelOfDetail = EffectDetailLevel.Low;
	}

	public override void Present()
	{
		if (NativeMethods.LogiLedSetTargetDevice(2))
		{
			Vector4 processedLedColor = GetProcessedLedColor(0);
			NativeMethods.LogiLedSetLighting((int)(processedLedColor.X * 100f), (int)(processedLedColor.Y * 100f), (int)(processedLedColor.Z * 100f));
		}
	}
}
