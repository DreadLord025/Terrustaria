using ReLogic.Content.Sources;

namespace ReLogic.Content;

public delegate void ContentFileUpdated(IContentSource contentSource, string path, string fullPath);
