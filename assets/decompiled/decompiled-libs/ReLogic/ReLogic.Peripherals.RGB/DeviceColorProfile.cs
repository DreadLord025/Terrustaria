using Microsoft.Xna.Framework;
using Newtonsoft.Json;

namespace ReLogic.Peripherals.RGB;

public class DeviceColorProfile
{
	private Vector3 _multiplier;

	[JsonProperty("R")]
	private float RedMultiplier
	{
		get
		{
			return _multiplier.X;
		}
		set
		{
			_multiplier.X = value;
		}
	}

	[JsonProperty("G")]
	private float GreenMultiplier
	{
		get
		{
			return _multiplier.Y;
		}
		set
		{
			_multiplier.Y = value;
		}
	}

	[JsonProperty("B")]
	private float BlueMultiplier
	{
		get
		{
			return _multiplier.Z;
		}
		set
		{
			_multiplier.Z = value;
		}
	}

	public DeviceColorProfile()
	{
		_multiplier = Vector3.One;
	}

	public DeviceColorProfile(Vector3 multiplier)
	{
		_multiplier = multiplier;
	}

	public void Apply(ref Vector4 color)
	{
		color.X *= _multiplier.X;
		color.Y *= _multiplier.Y;
		color.Z *= _multiplier.Z;
	}

	public void Apply(ref Vector3 color)
	{
		color.X *= _multiplier.X;
		color.Y *= _multiplier.Y;
		color.Z *= _multiplier.Z;
	}
}
