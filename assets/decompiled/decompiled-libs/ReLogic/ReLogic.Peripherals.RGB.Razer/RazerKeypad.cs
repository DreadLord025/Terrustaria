using Microsoft.Xna.Framework;

namespace ReLogic.Peripherals.RGB.Razer;

internal class RazerKeypad(DeviceColorProfile colorProfile) : RgbDevice(RgbDeviceVendor.Razer, RgbDeviceType.Keypad, Fragment.FromGrid(new Rectangle(-10, 0, 5, 4)), colorProfile)
{
	private NativeMethods.CustomKeypadEffect _effect = NativeMethods.CustomKeypadEffect.Create();

	private readonly EffectHandle _handle = new EffectHandle();

	public override void Present()
	{
		for (int i = 0; i < LedCount; i++)
		{
			_effect.Color[i] = RazerHelper.Vector4ToDeviceColor(GetProcessedLedColor(i));
		}
		_handle.SetAsKeypadEffect(ref _effect);
		_handle.Apply();
	}
}
