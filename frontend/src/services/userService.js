
const coopLocation = "39.560676588034035, -84.40952047456152"


async function getData() {
  const url = "http://192.168.8.110:3000/readings/latest";
  try {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Response status: ${response.status}`);
    }

    const result = await response.json();
    console.log(result);
  } catch (error) {
    console.error(error.message);
  }
}

export default getData;