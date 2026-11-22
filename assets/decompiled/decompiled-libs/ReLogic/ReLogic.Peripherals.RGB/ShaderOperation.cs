namespace ReLogic.Peripherals.RGB;

internal struct ShaderOperation(ChromaShader shader, ShaderBlendState blendState, EffectDetailLevel detailLevel)
{
	public readonly ShaderBlendState BlendState = blendState;

	private readonly ChromaShader _shader = shader;

	private readonly EffectDetailLevel _detailLevel = detailLevel;

	public void Process(RgbDevice device, Fragment fragment, float time)
	{
		_shader.Process(device, fragment, _detailLevel, time);
	}

	public ShaderOperation WithBlendState(ShaderBlendState blendState)
	{
		return new ShaderOperation(_shader, blendState, _detailLevel);
	}
}
