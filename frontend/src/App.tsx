import Card from "./components/Card";

function App() {
  return (
    <div className="container text-center">
      <div className="row">
        <div className="col">
          <Card title={"Temperature"} desc={"hot as hell"}></Card>
        </div>
        <div className="col">
          <Card title={"Humidity"} desc={"test"}></Card>
        </div>
        <div className="col">
          <Card title={"Ammonia"} desc={"test"}></Card>
        </div>
      </div>
    </div>
  );
}

export default App;
