import * as foo from "./graphics.js";

foo.bar();


/* //bootstrap
import("./index.js")
  .catch(e => console.error("Error importing `index.js`:", e)); */


/* //index
import * as foo from "../pkg/new/graphics.js";
import resourcesJSON from "../pkg/new/resources.json";

console.log(resourcesJSON.BALL);
//foo.bar(); */


