using Microsoft.Xna.Framework;

namespace ReLogic.Peripherals.RGB.Razer;

internal class RazerMouse(DeviceColorProfile colorProfile) : RgbDevice(RgbDeviceVendor.Razer, RgbDeviceType.Mouse, Fragment.FromGrid(new Rectangle(27, 0, 7, 9)), colorProfile)
{
	private NativeMethods.CustomMouseEffect _effect = NativeMethods.CustomMouseEffect.Create();

	private readonly EffectHandle _handle = new EffectHandle();

	public override void Present()
	{
		for (int i = 0; i < LedCount; i++)
		{
			_effect.Color[i] = RazerHelper.Vector4ToDeviceColor(GetProcessedLedColor(i));
		}
		_handle.SetAsMouseEffect(ref _effect);
		_handle.Apply();
	}
}
