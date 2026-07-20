import Alpine from "alpinejs";

const getErrorMessage = (target: HTMLInputElement): string | null => {
	const validity = target.validity;
	if (validity.tooLong) return "Field is too long";
	if (validity.tooShort) return "Field is too short";
	if (validity.valueMissing) return "field is required";
	if (validity.customError) return target.validationMessage;
	return null;
};

export type ErrorObject = {
	code: number;
	message: string;
	target: string;
	details: {
		code: number;
		message: string;
		target: string;
	}[];
};

export const initSharedAlpine = () => {
	Alpine.data("field",()=>({
		errors: [] as string[],
		onInput(){
			const field = this.$refs.input as HTMLInputElement;
			if(!field) return;
			this.errors = [];
			if(!field.checkValidity()){
				field.setAttribute("aria-invalid", "true");
				const message = getErrorMessage(field);
				if (!message) return;
				this.errors.push(message);
			} else {
				field.removeAttribute("aria-invalid");
				this.errors = [];
			}
		}
	}));
}

