using System;
using System.Diagnostics;
using System.IO;
using ReLogic.OS.Base;

namespace ReLogic.OS.Linux;

internal class PathService : ReLogic.OS.Base.PathService
{
	public override string GetStoragePath()
	{
		string text = Environment.GetEnvironmentVariable("XDG_DATA_HOME");
		if (string.IsNullOrEmpty(text))
		{
			text = Environment.GetEnvironmentVariable("HOME");
			if (string.IsNullOrEmpty(text))
			{
				return ".";
			}
			text += "/.local/share";
		}
		return text;
	}

	public override void OpenURL(string url)
	{
		Process.Start("xdg-open", "\"" + url + "\"");
	}

	public override void MoveToRecycleBin(string path)
	{
		string text = Environment.GetEnvironmentVariable("XDG_DATA_HOME");
		if (string.IsNullOrEmpty(text))
		{
			text = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".local", "share");
		}
		string text2 = Path.Combine(text, "Trash", "files");
		string text3 = Path.Combine(text, "Trash", "info");
		Directory.CreateDirectory(text2);
		Directory.CreateDirectory(text3);
		string fileName = Path.GetFileName(path);
		string text4 = Path.Combine(text2, fileName);
		int num = 1;
		while (File.Exists(text4))
		{
			text4 = Path.Combine(text2, Path.GetFileNameWithoutExtension(fileName) + "_" + num++ + Path.GetExtension(fileName));
		}
		string path2 = Path.Combine(text3, Path.GetFileName(text4) + ".trashinfo");
		string text5 = DateTime.UtcNow.ToString("yyyy-MM-ddTHH:mm:ss");
		File.WriteAllText(path2, "[Trash Info]\nPath=" + path + "\nDeletionDate=" + text5 + "\n");
		File.Move(path, text4);
	}
}
