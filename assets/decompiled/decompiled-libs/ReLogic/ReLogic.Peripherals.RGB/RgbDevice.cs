using Microsoft.Xna.Framework;
using ReLogic.Graphics;

namespace ReLogic.Peripherals.RGB;

public abstract class RgbDevice
{
	public readonly RgbDeviceType Type;

	public readonly RgbDeviceVendor Vendor;

	private readonly Fragment _backBuffer;

	private readonly Fragment _workingFragment;

	private readonly DeviceColorProfile _colorProfile;

	public EffectDetailLevel PreferredLevelOfDetail { get; protected set; }

	public int LedCount => _backBuffer.Count;

	protected RgbDevice(RgbDeviceVendor vendor, RgbDeviceType type, Fragment fragment, DeviceColorProfile colorProfile)
	{
		PreferredLevelOfDetail = EffectDetailLevel.High;
		Vendor = vendor;
		Type = type;
		_backBuffer = fragment;
		_workingFragment = fragment.CreateCopy();
		_colorProfile = colorProfile;
	}

	public virtual Fragment Rasterize()
	{
		return _workingFragment;
	}

	protected Vector4 GetProcessedLedColor(int index)
	{
		Vector4 color = _backBuffer.Colors[index];
		_colorProfile.Apply(ref color);
		return color;
	}

	protected Vector4 GetUnprocessedLedColor(int index)
	{
		return _backBuffer.Colors[index];
	}

	protected Color ProcessLedColor(Color color)
	{
		Vector3 color2 = color.ToVector3();
		_colorProfile.Apply(ref color2);
		return new Color(color2);
	}

	public void SetLedColor(int index, Vector4 color)
	{
		_backBuffer.Colors[index] = color;
	}

	public Vector2 GetLedCanvasPosition(int index)
	{
		return _backBuffer.GetCanvasPositionOfIndex(index);
	}

	public Point GetLedGridPosition(int index)
	{
		return _backBuffer.GetGridPositionOfIndex(index);
	}

	public virtual void Render(Fragment fragment, ShaderBlendState blendState)
	{
		switch (blendState.Mode)
		{
		case BlendMode.PerPixelOpacity:
		{
			for (int j = 0; j < fragment.Count; j++)
			{
				_backBuffer.Colors[j] = Vector4.Lerp(_backBuffer.Colors[j], fragment.Colors[j], blendState.GlobalOpacity * fragment.Colors[j].W);
			}
			break;
		}
		case BlendMode.GlobalOpacityOnly:
		{
			for (int k = 0; k < fragment.Count; k++)
			{
				_backBuffer.Colors[k] = Vector4.Lerp(_backBuffer.Colors[k], fragment.Colors[k], blendState.GlobalOpacity);
			}
			break;
		}
		default:
		{
			for (int i = 0; i < fragment.Count; i++)
			{
				_backBuffer.Colors[i] = fragment.Colors[i];
			}
			break;
		}
		}
	}

	public abstract void Present();

	public virtual void DebugDraw(IDebugDrawer drawer, Vector2 position, float scale)
	{
		for (int i = 0; i < LedCount; i++)
		{
			Vector2 canvasPositionOfIndex = _backBuffer.GetCanvasPositionOfIndex(i);
			Vector4 vector = _backBuffer.Colors[i];
			vector.X *= vector.W;
			vector.Y *= vector.W;
			vector.Z *= vector.W;
			drawer.DrawSquare(new Vector4(canvasPositionOfIndex * scale + position, scale / 10f, scale / 10f), new Color(vector));
		}
	}
}
