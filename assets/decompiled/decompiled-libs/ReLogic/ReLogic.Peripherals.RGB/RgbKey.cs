using System;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Input;

namespace ReLogic.Peripherals.RGB;

public class RgbKey
{
	public readonly Keys Key;

	public readonly string KeyTriggerName;

	private float _timeRemaining;

	private float _totalTime = 1f;

	private float _effectRate = 1f;

	public int CurrentIntegerRepresentation { get; private set; }

	public RgbKeyEffect Effect { get; private set; }

	public Color BaseColor { get; private set; }

	public Color TargetColor { get; private set; }

	public Color CurrentColor { get; private set; }

	public bool IsVisible => Effect != RgbKeyEffect.Clear;

	public RgbKey(Keys key, string keyTriggerName)
	{
		Key = key;
		KeyTriggerName = keyTriggerName;
		BaseColor = Color.White;
		TargetColor = Color.White;
		CurrentColor = Color.White;
		Effect = RgbKeyEffect.Clear;
	}

	public void SetIntegerRepresentation(int integerValue)
	{
		CurrentIntegerRepresentation = integerValue;
	}

	public void FadeTo(Color targetColor, float time)
	{
		TargetColor = targetColor;
		_timeRemaining = time;
		_totalTime = time;
		Effect = RgbKeyEffect.Fade;
	}

	public void SetFlashing(Color flashColor, float time, float flashesPerSecond = 1f)
	{
		TargetColor = flashColor;
		_timeRemaining = time;
		_totalTime = time;
		_effectRate = flashesPerSecond;
		Effect = RgbKeyEffect.Flashing;
	}

	public void SetFlashing(Color baseColor, Color flashColor, float time, float flashesPerSecond = 1f)
	{
		SetBaseColor(baseColor);
		SetFlashing(flashColor, time, flashesPerSecond);
	}

	public void SetBaseColor(Color color)
	{
		BaseColor = color;
	}

	public void SetTargetColor(Color color)
	{
		TargetColor = color;
	}

	public void SetSolid()
	{
		Effect = RgbKeyEffect.Solid;
	}

	public void SetSolid(Color color)
	{
		BaseColor = color;
		Effect = RgbKeyEffect.Solid;
	}

	public void Clear()
	{
		Effect = RgbKeyEffect.Clear;
	}

	internal void Update(float timeElapsed)
	{
		switch (Effect)
		{
		case RgbKeyEffect.Solid:
			UpdateSolidEffect();
			break;
		case RgbKeyEffect.Fade:
			UpdateFadeEffect();
			break;
		case RgbKeyEffect.Flashing:
			UpdateFlashingEffect();
			break;
		}
		_timeRemaining = Math.Max(_timeRemaining - timeElapsed, 0f);
	}

	private void UpdateSolidEffect()
	{
		CurrentColor = BaseColor;
	}

	private void UpdateFadeEffect()
	{
		float amount = 0f;
		if (_totalTime > 0f)
		{
			amount = 1f - _timeRemaining / _totalTime;
		}
		CurrentColor = Color.Lerp(BaseColor, TargetColor, amount);
	}

	private void UpdateFlashingEffect()
	{
		float amount = (float)Math.Sin(_timeRemaining * _effectRate * ((float)Math.PI * 2f)) * 0.5f + 0.5f;
		CurrentColor = Color.Lerp(BaseColor, TargetColor, amount);
	}
}
