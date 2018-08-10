import { keys } from 'lodash';

export interface TypedEventEmitter<T> {
  on<K extends keyof T>(event: K, listener: (arg: T[K]) => any): this;
  emit<K extends keyof T>(event: K, arg: T[K]): this;
  removeAllListeners<K extends keyof T>(event?: K): this;
  removeListener<K extends keyof T>(event: K, listener: (arg: T[K]) => any): this;
}

export type Listener<T, K extends keyof T> = (arg: T[K]) => any;

export class EventEmitter<T> implements TypedEventEmitter<T> {
  private readonly listeners: { [key in keyof T]?: Array<Listener<T, any>> };

  constructor() {
    this.listeners = {};
  }

  emit<K extends keyof T>(event: K, arg: T[K]) {

    const listeners = this.getListeners(event);
    if (listeners === undefined) {
      return this;
    }

    for (const listener of listeners) {
      listener(arg);
    }

    return this;
  }

  on<K extends keyof T>(event: K, listener: Listener<T, K>) {
    this.requireListeners(event).push(listener);
    return this;
  }

  removeAllListeners<K extends keyof T>(event?: K) {

    if (event) {
      delete this.listeners[event];
    } else {
      for (const key of keys(this.listeners)) {
        delete this.listeners[key];
      }
    }

    return this;
  }

  removeListener<K extends keyof T>(event: K, listener: Listener<T, K>) {

    const listeners = this.getListeners(event);
    if (listeners === undefined) {
      return this;
    }

    const i = listeners.indexOf(listener);
    if (i >= 0) {
      listeners.splice(i, 1);
    }

    return this;
  }

  private getListeners<K extends keyof T>(event: K): Array<Listener<T, K>> | undefined {
    return this.listeners[event];
  }

  private requireListeners<K extends keyof T>(event: K): Array<Listener<T, K>> {

    const listeners: Array<Listener<T, K>> | undefined = this.listeners[event];
    if (listeners !== undefined) {
      return listeners;
    }

    const newListeners: Array<Listener<T, K>> = [];
    this.listeners[event] = newListeners;

    return newListeners;
  }
}
