import { GroupNameExistsValidator } from './group-name-exists.directive';

describe('GroupNameExistsDirective', () => {
  it('should create an instance', () => {
    const directive = new GroupNameExistsValidator();
    expect(directive).toBeTruthy();
  });
});
