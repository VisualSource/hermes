import Alpine from "alpinejs";
import FieldValidator from "./field";

document.addEventListener("alpine:init",()=>{
    Alpine.data("field",FieldValidator);
});
