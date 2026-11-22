namespace ReLogic.Peripherals.RGB;

public struct ShaderBlendState(BlendMode blendMode, float alpha = 1f)
{
	public readonly float GlobalOpacity = alpha;

	public readonly BlendMode Mode = blendMode;
}
