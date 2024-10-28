import { Directive, forwardRef } from '@angular/core';
import { AbstractControl, AsyncValidator, NG_ASYNC_VALIDATORS, ValidationErrors } from '@angular/forms';
import { invoke } from '@tauri-apps/api/core';
import { from, map, Observable, of, switchMap, timer } from 'rxjs';

@Directive({
  selector: '[group-name-exists]',
  providers: [
    {
      provide: NG_ASYNC_VALIDATORS,
      useExisting: forwardRef(() => GroupNameExistsValidator),
      multi: true,
    },
  ],
})

export class GroupNameExistsValidator implements AsyncValidator {
  constructor() {}

  validate(control: AbstractControl): Observable<ValidationErrors | null> {
    if (!control.value) {
      return of(null); // No validation needed if the field is empty
    }
    return timer(300).pipe(
      switchMap(() => from(invoke("group_name_exists", {name: control.value}))),
      map(exists => exists === true ? {groupNameExists: true} : null)
    ) 
  }
}
