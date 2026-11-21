/** Every failure in this app should produce an explanation, a probable
 * cause, and a suggested action -- never a raw error alone. */
export function ErrorNotice({
  explanation,
  probableCause,
  suggestedAction,
}: {
  explanation: string;
  probableCause?: string;
  suggestedAction?: string;
}) {
  return (
    <div
      role="alert"
      className="border border-red-300 bg-red-50 rounded p-3 text-sm text-red-900"
    >
      <p className="font-medium">{explanation}</p>
      {probableCause && (
        <p className="mt-1">
          <span className="font-medium">Wahrscheinliche Ursache: </span>
          {probableCause}
        </p>
      )}
      {suggestedAction && (
        <p className="mt-1">
          <span className="font-medium">Vorschlag: </span>
          {suggestedAction}
        </p>
      )}
    </div>
  );
}
