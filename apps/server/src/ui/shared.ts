declare const grecaptcha: {
	enterprise: {
		ready: (callback: () => void) => void;
		execute: (key: string, opt?: { action: string }) => Promise<string>;
	};
};

type AlpineThis = {
	$refs: Record<string,HTMLElement>
}

declare const Alpine: {
	data: (name: string, state: ()=>Record<string,unknown>) => void;
}

document.addEventListener("alpine:init",()=>{
	Alpine.data("field",()=>({
		errors: [],
		onInput(this: AlpineThis & { errors: string[] }){
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
});

export const getErrorMessage = (target: HTMLInputElement): string | null => {
	const validity = target.validity;
	if (validity.tooLong) return "Field is too long";
	if (validity.tooShort) return "Field is too short";
	if (validity.valueMissing) return "field is required";
	if (validity.customError) return target.validationMessage;
	return null;
};


type ErrorObject = {
	code: number;
	message: string;
	target: string;
	details: {
		code: number;
		message: string;
		target: string;
	}[];
};

export const showMessage = (message: string) => {
	const dialog = document.getElementById(
		"message-dialog",
	) as HTMLDialogElement | null;
	if (!dialog) throw new Error("failed to get dialog");

	const content = dialog.querySelector("p");
	if (!content) throw new Error("Failed to find content element");

	content.textContent = message;

	dialog.showModal();
};

export const handleErrorResponse = async (response: Response) => {
	const body = (await response.json()) as ErrorObject;

	switch (response.status) {
		case 400:
			for (const err of body.details) {
	

			}

			break;
		case 500:
		case 403:
			showMessage(body.message);
			break;
	}
};

export const getRecaptchaToken = async (siteKey: string, action: string) => {
	await new Promise<void>((ok) => grecaptcha.enterprise.ready(ok));

	return await grecaptcha.enterprise.execute(siteKey, {
		action,
	});
};
