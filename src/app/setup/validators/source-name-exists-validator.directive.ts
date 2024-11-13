import { Directive, forwardRef } from '@angular/core';
import { AbstractControl, AsyncValidator, NG_ASYNC_VALIDATORS, ValidationErrors } from '@angular/forms';
import { invoke } from '@tauri-apps/api/core';
import { from, map, Observable, of, switchMap, timer } from 'rxjs';

@Directive({
  selector: '[source-name-exists]',
  providers: [
    {
      provide: NG_ASYNC_VALIDATORS,
      useExisting: forwardRef(() => SourceNameExistsValidator),
      multi: true,
    },
  ],
})

export class SourceNameExistsValidator implements AsyncValidator {
  constructor() {}

  validate(control: AbstractControl): Observable<ValidationErrors | null> {
    let value = control.value?.trim();
    if (!value) {
      return of(null); // No validation needed if the field is empty
    }
    return timer(300).pipe(
      switchMap(() => from(invoke("source_name_exists", {name: value}))),
      map(exists => exists === true ? {sourceNameExists: true} : null)
    ) 
  }
}
