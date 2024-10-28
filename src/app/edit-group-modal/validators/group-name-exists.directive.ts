import { Directive, forwardRef, Input } from '@angular/core';
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
  @Input()
  sourceId?: number;
  @Input()
  originalName?: string;

  constructor() { }

  validate(control: AbstractControl): Observable<ValidationErrors | null> {
    if (!control.value || (this.originalName && control.value == this.originalName)) {
      return of(null); // No validation needed if the field is empty
    }
    return timer(300).pipe(
      switchMap(() => from(invoke("group_exists", { name: control.value, sourceId: this.sourceId }))),
      map(exists => exists === true ? { groupNameExists: true } : null)
    )
  }
}
