// Shared layout primitives for all right-side panels.

export function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="review-section">
      <div className="review-section__title">{title}</div>
      <div className="review-section__body">{children}</div>
    </div>
  );
}

export function TagList({ tags }: { tags: string[] }) {
  return (
    <div className="review-tags">
      {tags.map((t) => (
        <span key={t} className="review-tag">
          {t}
        </span>
      ))}
    </div>
  );
}
