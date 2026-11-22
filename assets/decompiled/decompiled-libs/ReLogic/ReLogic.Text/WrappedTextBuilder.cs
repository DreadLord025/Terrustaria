using System.Globalization;
using System.IO;
using System.Text;

namespace ReLogic.Text;

public class WrappedTextBuilder
{
	private struct NonBreakingText
	{
		public readonly string Text;

		public readonly float Width;

		public readonly float WidthOnNewLine;

		public readonly bool IsWhitespace;

		private IFontMetrics _font;

		public NonBreakingText(IFontMetrics font, string text)
		{
			Text = text;
			IsWhitespace = true;
			float num = 0f;
			float num2 = 0f;
			_font = font;
			for (int i = 0; i < text.Length; i++)
			{
				GlyphMetrics characterMetrics = font.GetCharacterMetrics(text[i]);
				if (i == 0)
				{
					num2 = characterMetrics.KernedWidthOnNewLine - characterMetrics.KernedWidth;
				}
				else
				{
					num += font.CharacterSpacing;
				}
				num += characterMetrics.KernedWidth;
				if (text[i] != ' ')
				{
					IsWhitespace = false;
				}
			}
			Width = num;
			WidthOnNewLine = num + num2;
		}
	}

	private readonly IFontMetrics _font;

	private readonly CultureInfo _culture;

	private readonly float _maxWidth;

	private readonly StringBuilder _completedText = new StringBuilder();

	private readonly StringBuilder _workingLine = new StringBuilder();

	private float _workingLineWidth;

	private bool _firstLine;

	public WrappedTextBuilder(IFontMetrics font, CultureInfo culture, float maxWidth, float firstLineOffset = 0f)
	{
		_font = font;
		_maxWidth = maxWidth;
		_culture = culture;
		_workingLineWidth = firstLineOffset;
		_firstLine = true;
	}

	private void CommitWorkingLine()
	{
		if (!_firstLine)
		{
			_completedText.Append('\n');
		}
		_workingLineWidth = 0f;
		_completedText.Append(_workingLine);
		_workingLine.Clear();
		_firstLine = false;
	}

	private void FinishPartialLine()
	{
		if (_workingLineWidth > 0f)
		{
			CommitWorkingLine();
		}
	}

	private void Append(NonBreakingText textToken)
	{
		float num = ((_workingLineWidth != 0f) ? (_workingLineWidth + _font.CharacterSpacing + textToken.Width) : textToken.WidthOnNewLine);
		if (num <= _maxWidth)
		{
			_workingLine.Append(textToken.Text);
			_workingLineWidth = num;
			return;
		}
		FinishPartialLine();
		if (!textToken.IsWhitespace)
		{
			if (textToken.WidthOnNewLine <= _maxWidth)
			{
				_workingLine.Append(textToken.Text);
				_workingLineWidth = textToken.WidthOnNewLine;
			}
			else
			{
				AppendWithHardBreaks(textToken.Text);
			}
		}
	}

	private void AppendWithHardBreaks(string text)
	{
		for (int i = 0; i < text.Length; i++)
		{
			char c = text[i];
			GlyphMetrics characterMetrics = _font.GetCharacterMetrics(c);
			float num = ((!_workingLine.IsEmpty()) ? (_workingLineWidth + _font.CharacterSpacing + characterMetrics.KernedWidth) : characterMetrics.KernedWidthOnNewLine);
			if (num <= _maxWidth)
			{
				_workingLine.Append(c);
				_workingLineWidth = num;
			}
			else if (_workingLine.Length > 1 && !StringReaderWrapExtension.BreaksBetweenMostGlyphs(_culture))
			{
				_workingLineWidth += _font.CharacterSpacing + _font.GetCharacterMetrics('-').KernedWidth;
				while (_workingLine.Length > 1 && _workingLineWidth > _maxWidth)
				{
					_workingLineWidth -= _font.CharacterSpacing + _font.GetCharacterMetrics(_workingLine[_workingLine.Length - 1]).KernedWidth;
					_workingLine.Remove(_workingLine.Length - 1, 1);
					i--;
				}
				_workingLine.Append('-');
				FinishPartialLine();
				i--;
			}
			else
			{
				FinishPartialLine();
				_workingLine.Append(c);
				_workingLineWidth = characterMetrics.KernedWidthOnNewLine;
			}
		}
	}

	public string Build(string text)
	{
		StringReader stringReader = new StringReader(text);
		_completedText.EnsureCapacity(_completedText.Capacity + text.Length);
		while (stringReader.Peek() > 0)
		{
			if ((ushort)stringReader.Peek() == 10)
			{
				stringReader.Read();
				CommitWorkingLine();
			}
			else
			{
				string text2 = stringReader.ReadUntilBreakable(_culture);
				Append(new NonBreakingText(_font, text2));
			}
		}
		CommitWorkingLine();
		return _completedText.ToString();
	}
}
