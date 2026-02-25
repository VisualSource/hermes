declare const grecaptcha: {
	enterprise: {
		ready: (callback: () => void) => void;
		execute: (key: string, opt?: { action: string }) => Promise<string>;
	};
};

export const getErrorMessage = (target: HTMLInputElement): string | null => {
	const validity = target.validity;
	if (validity.tooLong) return "Field is too long";
	if (validity.tooShort) return "Field is too short";
	if (validity.valueMissing) return "field is required";
	if (validity.customError) return target.validationMessage;
	return null;
};

const getErrorReporter = (field: string) => {
	const reporter = document.getElementById(`${field}-errors`);
	if (!reporter) throw new Error("failed to find reporter");
	const list = reporter.querySelector("ul");
	if (!list) throw new Error("failed to get reportes list");

	return reporter;
};

const addError = (message: string, target: HTMLElement) => {
	const error = document.createElement("li");
	error.setAttribute("class", "text-xs text-left");
	error.textContent = message;

	target.replaceChildren(error);
};

export const reportError = (targets: string[]) => {
	for (const name of targets) {
		const target = document.getElementById(name) as HTMLInputElement | null;
		if (!target) continue;

		const reporter = getErrorReporter(target.name);

		target.addEventListener("invalid", (ev) => ev.preventDefault());
		target.addEventListener("input", () => {
			if (!target.checkValidity()) {
				reporter.classList.remove("hidden");
				target.setAttribute("aria-invalid", "true");
				const message = getErrorMessage(target);
				if (!message) return;

				addError(message, target);
			} else {
				target.removeAttribute("aria-invalid");
				reporter.classList.add("hidden");
			}
		});
	}
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
				addError(err.message, getErrorReporter(err.target));
				document
					.getElementById(`${err.target}-errors`)
					?.classList.remove("hidden");
			}

			document.getElementById("body-errors")?.classList.remove("hidden");
			break;
		case 500:
		case 403:
			showMessage(body.message);
			break;
	}
};

export const onFormSubmit = (
	target: string,
	callback: (formData: FormData) => void,
) => {
	const form = document.getElementById(target) as HTMLFormElement | null;
	if (!form) throw new Error("failed to get form");
	form?.addEventListener("submit", (ev) => {
		ev.preventDefault();
		const formData = new FormData(ev.target as HTMLFormElement);
		callback(formData);
	});
};

export const getRecaptchaToken = async (siteKey: string, action: string) => {
	await new Promise<void>((ok) => grecaptcha.enterprise.ready(ok));

	return await grecaptcha.enterprise.execute(siteKey, {
		action,
	});
};
