import { initReactI18next } from "react-i18next";
import i18n from "i18next";
import translations from "./translations.json" with { type: "json" };

i18n.use(initReactI18next).init({
	resources: translations,
	lng: "en",
	fallbackLng: "en",

	interpolation: {
		escapeValue: false,
	},
});
