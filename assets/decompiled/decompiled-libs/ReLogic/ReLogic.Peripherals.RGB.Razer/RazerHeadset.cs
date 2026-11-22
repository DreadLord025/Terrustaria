using Microsoft.Xna.Framework;

namespace ReLogic.Peripherals.RGB.Razer;

internal class RazerHeadset : RgbDevice
{
	private NativeMethods.CustomHeadsetEffect _effect = NativeMethods.CustomHeadsetEffect.Create();

	private readonly EffectHandle _handle = new EffectHandle();

	public RazerHeadset(DeviceColorProfile colorProfile)
		: base(RgbDeviceVendor.Razer, RgbDeviceType.Headset, Fragment.FromGrid(new Rectangle(-5, 0, 5, 1)), colorProfile)
	{
		PreferredLevelOfDetail = EffectDetailLevel.Low;
	}

	public override void Present()
	{
		for (int i = 0; i < _effect.Color.Length; i++)
		{
			_effect.Color[i] = RazerHelper.Vector4ToDeviceColor(GetProcessedLedColor(i));
		}
		_handle.SetAsHeadsetEffect(ref _effect);
		_handle.Apply();
	}
}
