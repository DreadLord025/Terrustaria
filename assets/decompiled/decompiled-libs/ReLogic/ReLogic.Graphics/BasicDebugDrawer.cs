using System;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;

namespace ReLogic.Graphics;

public class BasicDebugDrawer : IDebugDrawer, IDisposable
{
	private SpriteBatch _spriteBatch;

	private Texture2D _texture;

	private bool _disposedValue;

	public BasicDebugDrawer(GraphicsDevice graphicsDevice)
	{
		_spriteBatch = new SpriteBatch(graphicsDevice);
		_texture = new Texture2D(graphicsDevice, 4, 4);
		Color[] array = new Color[16];
		for (int i = 0; i < array.Length; i++)
		{
			array[i] = Color.White;
		}
		_texture.SetData(array);
	}

	public void Begin(Matrix matrix)
	{
		_spriteBatch.Begin(SpriteSortMode.Deferred, null, null, null, null, null, matrix);
	}

	public void Begin()
	{
		_spriteBatch.Begin();
	}

	public void DrawSquare(Vector4 positionAndSize, Color color)
	{
		_spriteBatch.Draw(_texture, new Vector2(positionAndSize.X, positionAndSize.Y), null, color, 0f, Vector2.Zero, new Vector2(positionAndSize.Z, positionAndSize.W) / 4f, SpriteEffects.None, 1f);
	}

	public void DrawSquare(Vector2 position, Vector2 size, Color color)
	{
		_spriteBatch.Draw(_texture, position, null, color, 0f, Vector2.Zero, size / 4f, SpriteEffects.None, 1f);
	}

	public void DrawSquareFromCenter(Vector2 center, Vector2 size, float rotation, Color color)
	{
		_spriteBatch.Draw(_texture, center, null, color, rotation, new Vector2(2f, 2f), size / 4f, SpriteEffects.None, 1f);
	}

	public void DrawLine(Vector2 start, Vector2 end, float width, Color color)
	{
		Vector2 vector = end - start;
		float rotation = (float)Math.Atan2(vector.Y, vector.X);
		Vector2 vector2 = new Vector2(vector.Length(), width);
		_spriteBatch.Draw(_texture, start, null, color, rotation, new Vector2(0f, 2f), vector2 / 4f, SpriteEffects.None, 1f);
	}

	public void End()
	{
		_spriteBatch.End();
	}

	protected virtual void Dispose(bool disposing)
	{
		if (_disposedValue)
		{
			return;
		}
		if (disposing)
		{
			if (_spriteBatch != null)
			{
				_spriteBatch.Dispose();
				_spriteBatch = null;
			}
			if (_texture != null)
			{
				_texture.Dispose();
				_texture = null;
			}
		}
		_disposedValue = true;
	}

	public void Dispose()
	{
		Dispose(disposing: true);
	}
}
