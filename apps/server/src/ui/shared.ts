import Alpine from "alpinejs";

export const getErrorMessage = (target: HTMLInputElement): string | null => {
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

export type AppStore = {
	done: boolean;
	dialog: {
		title: string;
		desc: string;
	};
	showError(title: string, desc: string ): void;
	showDone(): void;
}


export const initSharedAlpine = () => {
		Alpine.store("app",{
			done: false,
			dialog: {
				title: "",
				desc: "",
			},
			showError(title: string, desc: string){
				this.dialog.title = title;
				this.dialog.desc = desc;
				(document.getElementById("message-dialog") as HTMLDialogElement).showModal();
			},
			showDone(){
				this.done = true;
			}
		} as AppStore);
	Alpine.data("field",()=>({
		errors: [] as string[],
		hasErrors: false,
		onInput(ev: InputEvent){
			const field = ev.target as HTMLInputElement;
			if(!field) return;
			this.errors = [];
			if(!field.checkValidity()){
				field.setAttribute("aria-invalid", "true");
				const message = getErrorMessage(field);
				if (!message) return;
				this.errors.push(message);
				this.hasErrors = true;
			} else {
				field.removeAttribute("aria-invalid");
				this.errors = [];
				this.hasErrors = false;
			}
		}
	}));
}

