import http from "http";
import querystring from "querystring";

const PORT = 5000;
const flowers = []; // In-memory storage for flowers

const server = http.createServer(async (req, res) => {
  // Handle CORS headers
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");

  if (req.method === "OPTIONS") {
    res.writeHead(204); // Preflight response
    return res.end();
  }

  if (req.url === "/api/flowers" && req.method === "GET") {
    // Handle GET /api/flowers
    res.writeHead(200, { "Content-Type": "application/json" });
    return res.end(JSON.stringify(flowers));
  }

  if (req.url === "/api/flowers" && req.method === "POST") {
    // Handle POST /api/flowers with URL-encoded form data
    let body = "";
    req.on("data", (chunk) => (body += chunk));
    req.on("end", () => {
      try {
        const flower = querystring.parse(body);
        if (
          typeof flower.name === "string" &&
          typeof flower.color === "string"
        ) {
          flowers.push(flower);
          res.writeHead(201, { "Content-Type": "application/json" });
          return res.end(JSON.stringify({ message: "Flower added", flower }));
        } else {
          res.writeHead(400, { "Content-Type": "application/json" });
          return res.end(JSON.stringify({ message: "Invalid flower data" }));
        }
      } catch {
        res.writeHead(400, { "Content-Type": "application/json" });
        return res.end(JSON.stringify({ message: "Invalid form data" }));
      }
    });
  } else {
    // Handle 404 for other routes
    res.writeHead(404, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ message: "Route not found" }));
  }
});

server.listen(PORT, () => {
  console.log(`Server running at http://localhost:${PORT}/`);
});
