using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;
using ReLogic.Text;

namespace ReLogic.Graphics;

public class DynamicSpriteFont : IFontMetrics
{
	private class SpriteCharacterData
	{
		public readonly Texture2D Texture;

		public readonly Rectangle Glyph;

		public readonly Rectangle Padding;

		public readonly Vector3 Kerning;

		public SpriteCharacterData(Texture2D texture, Rectangle glyph, Rectangle padding, Vector3 kerning)
		{
			Texture = texture;
			Glyph = glyph;
			Padding = padding;
			Kerning = kerning;
		}

		public GlyphMetrics ToGlyphMetric()
		{
			return GlyphMetrics.FromKerningData(Kerning.X, Kerning.Y, Kerning.Z);
		}
	}

	public delegate void DrawCharacter(Texture2D texture, Vector2 position, Rectangle sourceRect, Vector2 scale);

	private SpriteCharacterData[] _spriteCharacters;

	private SpriteCharacterData _defaultCharacterData;

	public readonly char DefaultCharacter;

	private readonly float _characterSpacing;

	private readonly int _lineSpacing;

	public float CharacterSpacing => _characterSpacing;

	public int LineSpacing => _lineSpacing;

	public DynamicSpriteFont(float spacing, int lineSpacing, char defaultCharacter)
	{
		_characterSpacing = spacing;
		_lineSpacing = lineSpacing;
		DefaultCharacter = defaultCharacter;
	}

	public bool IsCharacterSupported(char character)
	{
		if (character != '\n' && character != '\r')
		{
			if (character < _spriteCharacters.Length)
			{
				return _spriteCharacters[(uint)character] != null;
			}
			return false;
		}
		return true;
	}

	public bool AreCharactersSupported(IEnumerable<char> characters)
	{
		foreach (char character in characters)
		{
			if (!IsCharacterSupported(character))
			{
				return false;
			}
		}
		return true;
	}

	internal void SetPages(FontPage[] pages)
	{
		char c = pages.SelectMany((FontPage p) => p.Characters).Max();
		_spriteCharacters = new SpriteCharacterData[c + 1];
		foreach (FontPage fontPage in pages)
		{
			for (int num2 = 0; num2 < fontPage.Characters.Count; num2++)
			{
				_spriteCharacters[(uint)fontPage.Characters[num2]] = new SpriteCharacterData(fontPage.Texture, fontPage.Glyphs[num2], fontPage.Padding[num2], fontPage.Kerning[num2]);
				if (fontPage.Characters[num2] == DefaultCharacter)
				{
					_defaultCharacterData = _spriteCharacters[(uint)fontPage.Characters[num2]];
				}
			}
		}
	}

	internal void InternalDraw(string text, SpriteBatch spriteBatch, Vector2 startPosition, Color color, float rotation, Vector2 origin, ref Vector2 scale, SpriteEffects spriteEffects, float depth)
	{
		InternalDraw(text, spriteBatch, startPosition, color, rotation, origin, ref scale, spriteEffects, depth, null, null, null);
	}

	internal void InternalDraw(string text, SpriteBatch spriteBatch, Vector2 startPosition, Color color, float rotation, Vector2 origin, ref Vector2 scale, SpriteEffects spriteEffects, float depth, Vector2[] charOffsets, Color[] charColors, float[] charRotations)
	{
		if (rotation == 0f && spriteEffects == SpriteEffects.None && charOffsets == null && charColors == null && charRotations == null)
		{
			InternalDrawFast(text, spriteBatch, startPosition, color, origin, scale, depth);
			return;
		}
		Matrix matrix = Matrix.CreateTranslation((0f - origin.X) * scale.X, (0f - origin.Y) * scale.Y, 0f) * Matrix.CreateRotationZ(rotation);
		Vector2 zero = Vector2.Zero;
		Vector2 one = Vector2.One;
		bool flag = true;
		float x = 0f;
		if (spriteEffects != SpriteEffects.None)
		{
			Vector2 vector = MeasureString(text);
			if ((spriteEffects & SpriteEffects.FlipHorizontally) != 0)
			{
				x = vector.X * scale.X;
				one.X = -1f;
			}
			if ((spriteEffects & SpriteEffects.FlipVertically) != 0)
			{
				zero.Y = (vector.Y - (float)LineSpacing) * scale.Y;
				one.Y = -1f;
			}
		}
		zero.X = x;
		for (int i = 0; i < text.Length; i++)
		{
			char c = text[i];
			switch (c)
			{
			case '\n':
				zero.X = x;
				zero.Y += (float)LineSpacing * scale.Y * one.Y;
				flag = true;
				continue;
			case '\r':
				continue;
			}
			SpriteCharacterData characterData = GetCharacterData(c);
			Vector3 kerning = characterData.Kerning;
			Rectangle padding = characterData.Padding;
			if ((spriteEffects & SpriteEffects.FlipHorizontally) != 0)
			{
				padding.X -= padding.Width;
			}
			if ((spriteEffects & SpriteEffects.FlipVertically) != 0)
			{
				padding.Y = LineSpacing - characterData.Glyph.Height - padding.Y;
			}
			if (flag)
			{
				kerning.X = Math.Max(kerning.X, 0f);
			}
			else
			{
				zero.X += CharacterSpacing * scale.X * one.X;
			}
			zero.X += kerning.X * scale.X * one.X;
			Color color2 = color;
			if (charColors != null)
			{
				color2 = charColors[i];
			}
			float num = rotation;
			if (charRotations != null)
			{
				num += charRotations[i];
			}
			Vector2 position = zero;
			position.X += (float)padding.X * scale.X;
			position.Y += (float)padding.Y * scale.Y;
			if (charOffsets != null)
			{
				position += charOffsets[i];
			}
			Vector2.Transform(ref position, ref matrix, out position);
			position += startPosition;
			spriteBatch.Draw(characterData.Texture, position, characterData.Glyph, color2, num, Vector2.Zero, scale, spriteEffects, depth);
			zero.X += (kerning.Y + kerning.Z) * scale.X * one.X;
			flag = false;
		}
	}

	internal void InternalDrawFast(string text, SpriteBatch spriteBatch, Vector2 position, Color color, Vector2 origin, Vector2 scale, float depth)
	{
		position -= origin * scale;
		bool flag = true;
		float x = position.X;
		foreach (char c in text)
		{
			switch (c)
			{
			case '\n':
				position.X = x;
				position.Y += (float)LineSpacing * scale.Y;
				flag = true;
				continue;
			case '\r':
				continue;
			}
			SpriteCharacterData characterData = GetCharacterData(c);
			Vector3 kerning = characterData.Kerning;
			Rectangle padding = characterData.Padding;
			if (flag)
			{
				kerning.X = Math.Max(kerning.X, 0f);
			}
			else
			{
				position.X += CharacterSpacing * scale.X;
			}
			position.X += kerning.X * scale.X;
			Vector2 position2 = position;
			position2.X += (float)padding.X * scale.X;
			position2.Y += (float)padding.Y * scale.Y;
			spriteBatch.Draw(characterData.Texture, position2, characterData.Glyph, color, 0f, Vector2.Zero, scale, SpriteEffects.None, depth);
			position.X += (kerning.Y + kerning.Z) * scale.X;
			flag = false;
		}
	}

	public void DrawCustomFast(DrawCharacter draw, string text, Vector2 position, Vector2 scale)
	{
		bool flag = true;
		float x = position.X;
		foreach (char c in text)
		{
			switch (c)
			{
			case '\n':
				position.X = x;
				position.Y += (float)LineSpacing * scale.Y;
				flag = true;
				continue;
			case '\r':
				continue;
			}
			SpriteCharacterData characterData = GetCharacterData(c);
			Vector3 kerning = characterData.Kerning;
			Rectangle padding = characterData.Padding;
			if (flag)
			{
				kerning.X = Math.Max(kerning.X, 0f);
			}
			else
			{
				position.X += CharacterSpacing * scale.X;
			}
			position.X += kerning.X * scale.X;
			Vector2 position2 = position;
			position2.X += (float)padding.X * scale.X;
			position2.Y += (float)padding.Y * scale.Y;
			draw(characterData.Texture, position2, characterData.Glyph, scale);
			position.X += (kerning.Y + kerning.Z) * scale.X;
			flag = false;
		}
	}

	private SpriteCharacterData GetCharacterData(char character)
	{
		return ((character < _spriteCharacters.Length) ? _spriteCharacters[(uint)character] : null) ?? _defaultCharacterData;
	}

	public Vector2 MeasureString(string text)
	{
		if (text.Length == 0)
		{
			return Vector2.Zero;
		}
		Vector2 zero = Vector2.Zero;
		zero.Y = LineSpacing;
		float val = 0f;
		int num = 0;
		float num2 = 0f;
		bool flag = true;
		foreach (char c in text)
		{
			switch (c)
			{
			case '\n':
				val = Math.Max(zero.X + Math.Max(num2, 0f), val);
				num2 = 0f;
				zero = Vector2.Zero;
				zero.Y = LineSpacing;
				flag = true;
				num++;
				continue;
			case '\r':
				continue;
			}
			SpriteCharacterData characterData = GetCharacterData(c);
			Vector3 kerning = characterData.Kerning;
			if (flag)
			{
				kerning.X = Math.Max(kerning.X, 0f);
			}
			else
			{
				zero.X += CharacterSpacing + num2;
			}
			zero.X += kerning.X + kerning.Y;
			num2 = kerning.Z;
			zero.Y = Math.Max(zero.Y, characterData.Padding.Height);
			flag = false;
		}
		zero.X += Math.Max(num2, 0f);
		zero.Y += num * LineSpacing;
		zero.X = Math.Max(zero.X, val);
		return zero;
	}

	public string CreateWrappedText(string text, float scale, float maxWidth, float firstLineOffset, CultureInfo culture)
	{
		if (scale == 0f)
		{
			scale = 1f;
		}
		return CreateWrappedText(text, maxWidth / scale, firstLineOffset, culture);
	}

	public string CreateWrappedText(string text, float maxWidth, CultureInfo culture)
	{
		return CreateWrappedText(text, maxWidth, 0f, culture);
	}

	public string CreateWrappedText(string text, float maxWidth, float firstLineOffset, CultureInfo culture)
	{
		if (maxWidth <= 0f)
		{
			return text;
		}
		return new WrappedTextBuilder(this, culture, maxWidth, firstLineOffset).Build(text);
	}

	public string CreateCroppedText(string text, float maxWidth)
	{
		Vector2 vector = MeasureString(text);
		Vector2 vector2 = MeasureString("…");
		maxWidth -= vector2.X;
		if (maxWidth <= vector2.X)
		{
			return "…";
		}
		if (vector.X > maxWidth)
		{
			int num = 200;
			while (vector.X > maxWidth && text.Length > 1)
			{
				num--;
				if (num <= 0)
				{
					break;
				}
				text = text.Substring(0, text.Length - 1);
				if (text.Length == 1)
				{
					text = "";
					break;
				}
				vector = MeasureString(text);
			}
			text += "…";
		}
		return text;
	}

	public GlyphMetrics GetCharacterMetrics(char character)
	{
		return GetCharacterData(character).ToGlyphMetric();
	}
}
