function Card({ title, desc }) {
  return (
    <div className="frosted-card">
      <h1>{title}</h1>
      <p>{desc}</p>
    </div>
  );
}

export default Card;
