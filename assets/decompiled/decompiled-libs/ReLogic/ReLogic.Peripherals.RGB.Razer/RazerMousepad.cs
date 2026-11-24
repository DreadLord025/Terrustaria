using Microsoft.Xna.Framework;

namespace ReLogic.Peripherals.RGB.Razer;

internal class RazerMousepad(DeviceColorProfile colorProfile) : RgbDevice(RgbDeviceVendor.Razer, RgbDeviceType.Mousepad, Fragment.FromCustom(CreatePositionList()), colorProfile)
{
	private NativeMethods.CustomMousepadEffect _effect = NativeMethods.CustomMousepadEffect.Create();

	private readonly EffectHandle _handle = new EffectHandle();

	private static Point[] CreatePositionList()
	{
		Point[] array = new Point[15];
		Point point = new Point(26, 0);
		for (int i = 0; i < 5; i++)
		{
			array[i] = new Point(point.X, point.Y + i);
			array[14 - i] = new Point(point.X + 6, point.Y + i);
		}
		for (int j = 5; j < 10; j++)
		{
			array[j] = new Point(j - 5 + point.X + 1, point.Y + 5);
		}
		return array;
	}

	public override void Present()
	{
		for (int i = 0; i < LedCount; i++)
		{
			_effect.Color[i] = RazerHelper.Vector4ToDeviceColor(GetProcessedLedColor(i));
		}
		_handle.SetAsMousepadEffect(ref _effect);
		_handle.Apply();
	}
}
