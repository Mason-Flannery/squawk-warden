interface CardProps {
  icon: string;
  title: string;
  desc: string;
}

function Card({ icon, title, desc }: CardProps) {
  return (
    <div className="bg-white/30 backdrop-blur-md rounded-lg p-3">
      <div className="flex gap-x-2">
        <div className="flex-none">
          <img src={icon} height={50} width={36}></img>
        </div>
        <div className="flex-auto">
          <p className="text-4xl font-light">{title}</p>
        </div>
        <div className="flex-auto">
          <p className="font-thin">{desc}</p>
        </div>
      </div>
    </div>
  );
}

export default Card;
