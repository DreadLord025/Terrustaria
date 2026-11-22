namespace ReLogic.OS;

public interface IPathService
{
	string GetStoragePath();

	string GetStoragePath(string subfolder);

	string ExpandPathVariables(string path);

	void OpenURL(string url);

	void MoveToRecycleBin(string path);
}
