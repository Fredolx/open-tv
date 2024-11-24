import { Directive, forwardRef, Input, OnChanges, SimpleChanges } from '@angular/core';
import { AbstractControl, NG_VALIDATORS, ValidationErrors, Validator } from '@angular/forms';

@Directive({
  selector: '[empty]',
  providers: [
    {
      provide: NG_VALIDATORS,
      useExisting: forwardRef(() => NotEmptyValidatorDirective),
      multi: true
    }
  ]
})
export class NotEmptyValidatorDirective implements Validator, OnChanges {

  @Input('emptyDisabled')
  disabled = false;
  private onChange: (() => void) | undefined;

  constructor() {}

  validate(control: AbstractControl): ValidationErrors | null {
    if (this.disabled === true) 
      return null;
    if (!control.value?.trim()) {
      return { 'empty': true };
    }
    return null;
  }

  registerOnValidatorChange(fn: () => void): void {
    this.onChange = fn;
  }

  ngOnChanges(changes: SimpleChanges): void {
    if ('disabled' in changes && this.onChange) {
      this.onChange();
    }
  }
}
