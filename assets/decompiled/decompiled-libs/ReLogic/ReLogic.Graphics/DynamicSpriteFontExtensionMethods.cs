using System.Text;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;

namespace ReLogic.Graphics;

public static class DynamicSpriteFontExtensionMethods
{
	public static void DrawString(this SpriteBatch spriteBatch, DynamicSpriteFont spriteFont, string text, Vector2 position, Color color)
	{
		Vector2 scale = Vector2.One;
		spriteFont.InternalDraw(text, spriteBatch, position, color, 0f, Vector2.Zero, ref scale, SpriteEffects.None, 0f);
	}

	public static void DrawString(this SpriteBatch spriteBatch, DynamicSpriteFont spriteFont, StringBuilder text, Vector2 position, Color color)
	{
		Vector2 scale = Vector2.One;
		spriteFont.InternalDraw(text.ToString(), spriteBatch, position, color, 0f, Vector2.Zero, ref scale, SpriteEffects.None, 0f);
	}

	public static void DrawString(this SpriteBatch spriteBatch, DynamicSpriteFont spriteFont, string text, Vector2 position, Color color, float rotation, Vector2 origin, float scale, SpriteEffects effects, float layerDepth, Vector2[] charOffsets = null, Color[] charColors = null)
	{
		Vector2 scale2 = new Vector2
		{
			X = scale,
			Y = scale
		};
		spriteFont.InternalDraw(text, spriteBatch, position, color, rotation, origin, ref scale2, effects, layerDepth, charOffsets, charColors, null);
	}

	public static void DrawString(this SpriteBatch spriteBatch, DynamicSpriteFont spriteFont, StringBuilder text, Vector2 position, Color color, float rotation, Vector2 origin, float scale, SpriteEffects effects, float layerDepth)
	{
		Vector2 scale2 = new Vector2(scale);
		spriteFont.InternalDraw(text.ToString(), spriteBatch, position, color, rotation, origin, ref scale2, effects, layerDepth);
	}

	public static void DrawString(this SpriteBatch spriteBatch, DynamicSpriteFont spriteFont, string text, Vector2 position, Color color, float rotation, Vector2 origin, Vector2 scale, SpriteEffects effects, float layerDepth)
	{
		spriteFont.InternalDraw(text, spriteBatch, position, color, rotation, origin, ref scale, effects, layerDepth);
	}

	public static void DrawString(this SpriteBatch spriteBatch, DynamicSpriteFont spriteFont, StringBuilder text, Vector2 position, Color color, float rotation, Vector2 origin, Vector2 scale, SpriteEffects effects, float layerDepth)
	{
		spriteFont.InternalDraw(text.ToString(), spriteBatch, position, color, rotation, origin, ref scale, effects, layerDepth);
	}
}
