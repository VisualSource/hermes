export const getErrorMessage = (target) => {
    const validity = target.validity;
    if (validity.tooLong)
        return "Field is too long";
    if (validity.tooShort)
        return "Field is too short";
    if (validity.valueMissing)
        return "field is required";
    if (validity.customError)
        return target.validationMessage;
    return null;
};
const getErrorReporter = (field) => {
    const reporter = document.getElementById(`${field}-errors`);
    if (!reporter)
        throw new Error("failed to find reporter");
    const list = reporter.querySelector("ul");
    if (!list)
        throw new Error("failed to get reportes list");
    return reporter;
};
const addError = (message, target) => {
    const error = document.createElement("li");
    error.setAttribute("class", "text-xs text-left");
    error.textContent = message;
    target.replaceChildren(error);
};
export const reportError = (targets) => {
    for (const name of targets) {
        const target = document.getElementById(name);
        if (!target)
            continue;
        const reporter = getErrorReporter(target.name);
        target.addEventListener("invalid", (ev) => ev.preventDefault());
        target.addEventListener("input", () => {
            if (!target.checkValidity()) {
                reporter.classList.remove("hidden");
                target.setAttribute("aria-invalid", "true");
                const message = getErrorMessage(target);
                if (!message)
                    return;
                addError(message, target);
            }
            else {
                target.removeAttribute("aria-invalid");
                reporter.classList.add("hidden");
            }
        });
    }
};
export const showMessage = (message) => {
    const dialog = document.getElementById("message-dialog");
    if (!dialog)
        throw new Error("failed to get dialog");
    const content = dialog.querySelector("p");
    if (!content)
        throw new Error("Failed to find content element");
    content.textContent = message;
    dialog.showModal();
};
export const handleErrorResponse = async (response) => {
    const body = (await response.json());
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
export const onFormSubmit = (target, callback) => {
    const form = document.getElementById(target);
    if (!form)
        throw new Error("failed to get form");
    form?.addEventListener("submit", (ev) => {
        ev.preventDefault();
        const formData = new FormData(ev.target);
        callback(formData);
    });
};
export const getRecaptchaToken = async (siteKey, action) => {
    await new Promise((ok) => grecaptcha.enterprise.ready(ok));
    return await grecaptcha.enterprise.execute(siteKey, {
        action,
    });
};
//# sourceMappingURL=shared.js.map