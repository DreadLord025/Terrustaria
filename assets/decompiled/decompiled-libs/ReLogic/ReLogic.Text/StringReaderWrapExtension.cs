using System.Globalization;
using System.IO;
using System.Text;

namespace ReLogic.Text;

internal static class StringReaderWrapExtension
{
	private struct CharacterSet
	{
		private readonly bool[] arr = new bool[65535];

		public bool this[char c] => arr[(uint)c];

		public CharacterSet(string characters)
		{
			foreach (char c in characters)
			{
				arr[(uint)c] = true;
			}
		}
	}

	internal enum WrapScanMode
	{
		Space,
		NewLine,
		Word,
		None
	}

	private static readonly CharacterSet InvalidCharactersForLineStart = new CharacterSet("!%),.:;?]}¢°·'\"†‡›℃∶、。〃〆〕〗〞﹚﹜！＂％＇），．：；？！］｝～ \n!),.:;?]}¢·–— '\"• 、。〆〞〕〉》」︰︱︲\ufe33﹐﹑﹒\ufe53﹔﹕﹖﹘﹚﹜！），．：；？︶︸︺︼︾﹀﹂﹗］｜｝､");

	private static readonly CharacterSet InvalidCharactersForLineEnd = new CharacterSet("$(£¥·'\"〈《「『【〔〖〝﹙﹛＄（．［｛￡￥([{£¥'\"‵〈《「『〔〝\ufe34﹙﹛（｛︵︷︹︻︽︿﹁﹃\ufe4f");

	private static readonly CharacterSet WordTerminators = new CharacterSet("！，。、：？");

	private static readonly CharacterSet Numeric = new CharacterSet("0123456789,.");

	private static readonly CultureInfo SimplifiedChinese = new CultureInfo("zh-Hans");

	private static readonly CultureInfo TraditionalChinese = new CultureInfo("zh-Hant");

	internal static bool BreaksBetweenMostGlyphs(CultureInfo culture)
	{
		if (culture.LCID != SimplifiedChinese.LCID)
		{
			return culture.LCID == TraditionalChinese.LCID;
		}
		return true;
	}

	internal static bool IsIgnoredCharacter(char character)
	{
		if (character < ' ')
		{
			return character != '\n';
		}
		return false;
	}

	internal static bool CanBreakBetween(char previousChar, char nextChar, CultureInfo culture)
	{
		if (BreaksBetweenMostGlyphs(culture))
		{
			if (Numeric[previousChar] && Numeric[nextChar])
			{
				return false;
			}
			if (!InvalidCharactersForLineEnd[previousChar])
			{
				return !InvalidCharactersForLineStart[nextChar];
			}
			return false;
		}
		return WordTerminators[previousChar];
	}

	internal static WrapScanMode GetModeForCharacter(char character)
	{
		if (IsIgnoredCharacter(character))
		{
			return WrapScanMode.None;
		}
		switch (character)
		{
		case '\n':
			return WrapScanMode.NewLine;
		case ' ':
		case '\u2009':
		case '\u200a':
		case '\u200b':
			return WrapScanMode.Space;
		default:
			return WrapScanMode.Word;
		}
	}

	internal static string ReadUntilBreakable(this StringReader reader, CultureInfo culture)
	{
		StringBuilder stringBuilder = new StringBuilder();
		char c = (char)reader.Peek();
		WrapScanMode wrapScanMode = WrapScanMode.None;
		while (reader.Peek() > 0)
		{
			if (IsIgnoredCharacter((char)reader.Peek()))
			{
				reader.Read();
				continue;
			}
			char previousChar = c;
			c = (char)reader.Peek();
			WrapScanMode wrapScanMode2 = wrapScanMode;
			wrapScanMode = GetModeForCharacter(c);
			if (!stringBuilder.IsEmpty() && wrapScanMode2 != wrapScanMode)
			{
				return stringBuilder.ToString();
			}
			if (stringBuilder.IsEmpty())
			{
				stringBuilder.Append((char)reader.Read());
				continue;
			}
			if (CanBreakBetween(previousChar, c, culture))
			{
				return stringBuilder.ToString();
			}
			stringBuilder.Append((char)reader.Read());
		}
		return stringBuilder.ToString();
	}
}
