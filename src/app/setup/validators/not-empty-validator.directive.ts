import { Directive, forwardRef } from '@angular/core';
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
export class NotEmptyValidatorDirective implements Validator {

  constructor() {}

  validate(control: AbstractControl): ValidationErrors | null {
    if (!control.value?.trim()) {
      return { 'empty': true };
    }
    return null;
  }

}
