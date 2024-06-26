
export class Option<T> {

    constructor(private readonly value: T | undefined) {}
    
    to_optional(): T | undefined {
        return this.value;
    }

    is_some(): boolean {
        return this.value !== undefined;
    }

    is_some_cb(callback: (value: T) => void) {
        if (this.value !== undefined) {
            callback(this.value);
        }
    }

    is_none(): boolean {
        return this.value === undefined;
    }

    is_none_cb(callback: () => void) {
        if (this.value === undefined) {
            callback();
        }
    }

    unwrap(): T {
        if (this.value === undefined) {
            throw new Error("Cannot unwrap none option");
        }
        return this.value;
    }

    unwrap_or(default_value: T): T {
        if (this.value === undefined) {
            return default_value;
        }
        return this.value;
    }

}

export function Some<T>(value: T): Option<T> {
    return new Option(value);
}

export function None<T>(): Option<T> {
    return new Option<T>(undefined);
}