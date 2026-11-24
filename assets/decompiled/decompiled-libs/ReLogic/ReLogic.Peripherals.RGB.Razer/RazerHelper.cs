using Microsoft.Xna.Framework;

namespace ReLogic.Peripherals.RGB.Razer;

internal static class RazerHelper
{
	public static uint Vector4ToDeviceColor(Vector4 color)
	{
		int num = (int)(color.X * 255f);
		int num2 = (int)(color.Y * 255f);
		int num3 = (int)(color.Z * 255f);
		num3 <<= 16;
		num2 <<= 8;
		return (uint)(num | num2 | num3);
	}

	public static uint XnaColorToDeviceColor(Color color)
	{
		byte r = color.R;
		int g = color.G;
		int b = color.B;
		b <<= 16;
		g <<= 8;
		return (uint)(r | g | b);
	}
}
