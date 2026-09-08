interface CardProps {
  icon: string;
  title: string;
  desc: string;
}

function Card({ icon, title, desc: loc }: CardProps) {
  return (
    <div className="bg-white/30 backdrop-blur-md rounded-lg p-3">
      <div className="flex gap-x-2">
        <div className="flex-none">
          <img src={icon} height={50} width={36} className="items-center"></img>
        </div>
        <div className="flex-auto">
          <div>
            <p className="text-4xl font-light">{title}</p>
            <p className="font-thin">{loc}</p>
          </div>
        </div>
        <div className="flex-auto"></div>
      </div>
      <div>
        <p className="text-4xl font-light">21</p>
        <p className="font-thin">Cloudy Night</p>
      </div>
    </div>
  );
}

export default Card;
