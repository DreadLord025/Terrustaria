namespace ReLogic.Localization.IME;

internal class FnaIme : PlatformIme
{
	private bool _disposedValue;

	public override uint CandidateCount => 0u;

	public override string CompositionString => string.Empty;

	public override bool IsCandidateListVisible => false;

	public override uint? SelectedCandidate => 0u;

	private void OnCharCallback(char key)
	{
		if (IsEnabled)
		{
			OnKeyPress(key);
		}
	}

	public override string GetCandidate(uint index)
	{
		return string.Empty;
	}

	protected override void OnEnable()
	{
	}

	protected override void OnDisable()
	{
	}

	protected override void Dispose(bool disposing)
	{
		if (!_disposedValue)
		{
			if (IsEnabled)
			{
				Disable();
			}
			_disposedValue = true;
		}
	}

	~FnaIme()
	{
		Dispose(disposing: false);
	}
}
