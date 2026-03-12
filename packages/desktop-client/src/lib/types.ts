interface TypedEventTarget<T> extends EventTarget {
    addEventListener<K extends keyof T>(type: K, listener: (ev: T[K]) => void, options?: boolean | AddEventListenerOptions): void;
    addEventListener(
        type: string,
        callback: EventListenerOrEventListenerObject | null,
        options?: EventListenerOptions | boolean
    ): void;
    removeEventListener<K extends keyof T>(type: K,listener: (ev: T[K]) => void | null, options?: EventListenerOptions | boolean): void;
    removeEventListener(type: string, callback: EventListenerOrEventListenerObject | null, options?: EventListenerOptions | boolean): void;

    dispatchEvent(event: Event | T[keyof T]): boolean;
}

export interface TypedEmitter<T> {
    prototype: EventTarget;
    new(): TypedEventTarget<T>
}