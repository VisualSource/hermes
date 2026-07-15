export declare const getErrorMessage: (target: HTMLInputElement) => string | null;
export declare const reportError: (targets: string[]) => void;
export declare const showMessage: (message: string) => void;
export declare const handleErrorResponse: (response: Response) => Promise<void>;
export declare const onFormSubmit: (target: string, callback: (formData: FormData) => void) => void;
export declare const getRecaptchaToken: (siteKey: string, action: string) => Promise<string>;
//# sourceMappingURL=shared.d.ts.map