using System;
using System.IO;

namespace ReLogic.Content;

public static class AssetPathHelper
{
	public static string CleanPath(string path)
	{
		path = path.Replace('/', '\\');
		path = path.Replace("\\.\\", "\\");
		while (path.StartsWith(".\\"))
		{
			path = path.Substring(".\\".Length);
		}
		while (path.EndsWith("\\."))
		{
			path = ((path.Length <= "\\.".Length) ? "\\" : path.Substring(0, path.Length - "\\.".Length));
		}
		int num;
		for (num = 1; num < path.Length; num = CollapseParentDirectory(ref path, num, "\\..\\".Length))
		{
			num = path.IndexOf("\\..\\", num);
			if (num < 0)
			{
				break;
			}
		}
		if (path.EndsWith("\\.."))
		{
			int num2 = path.Length - "\\..".Length;
			if (num2 > 0)
			{
				CollapseParentDirectory(ref path, num2, "\\..".Length);
			}
		}
		if (path == ".")
		{
			path = string.Empty;
		}
		if (Path.DirectorySeparatorChar != '\\')
		{
			path = path.Replace('\\', Path.DirectorySeparatorChar);
		}
		return path;
	}

	private static int CollapseParentDirectory(ref string path, int position, int removeLength)
	{
		int num = path.LastIndexOf('\\', position - 1) + 1;
		path = path.Remove(num, position - num + removeLength);
		return Math.Max(num - 1, 1);
	}
}
