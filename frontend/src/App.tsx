import Card from "./components/Card";
import getData from "./services/userService.js";
import coopIcon from "./assets/light-mode/coop.svg";
import sunIcon from "./assets/light-mode/sun.svg";
import humidityIcon from "./assets/light-mode/humidity.svg";
import "./App.css";

const gradients = {
  morning:
    "bg-linear-to-br from-sunrise-left-blue via-sunrise-center-pink to-sunrise-right-yellow min-h-screen",
  afternoon:
    "bg-linear-to-br from-daytime-left-blue via-daytime-center-blue to-daytime-right-white min-h-screen",
  evening:
    "bg-linear-to-t from-evening-left-navy to-evening-right-purple min-h-screen",
  night: "idk",
};

function App() {
  return (
    <div className="bg-linear-to-br from-daytime-left-blue via-daytime-center-blue to-daytime-right-white min-h-screen">
      <div className="grid grid-cols-3 gap-3">
        <div>
          <Card
            icon={sunIcon}
            title={"Outside Temperature"}
            desc={"hot as hell"}
          ></Card>
        </div>
        <div>
          <Card icon={coopIcon} title={"Coop Temperature"} desc={"test"}></Card>
        </div>
        <div>
          <Card icon={humidityIcon} title={"Humidity"} desc={"test"}></Card>
        </div>
        <div>
          <button onClick={() => getData()}>Click Me</button>
        </div>
      </div>
    </div>
  );
}

export default App;
