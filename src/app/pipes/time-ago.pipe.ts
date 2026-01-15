import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
    name: 'timeAgo'
})
export class TimeAgoPipe implements PipeTransform {

    transform(value: any): string {
        if (!value) return '';

        const seconds = Math.floor((+new Date() - +new Date(value)) / 1000);

        if (seconds < 29) return 'Just now';

        const intervals: { [key: string]: number } = {
            'day': 86400,
            'hour': 3600,
            'minute': 60,
            'second': 1
        };

        let counter;
        for (const i in intervals) {
            counter = Math.floor(seconds / intervals[i]);
            if (counter > 0) {
                if (counter === 1) {
                    return counter + ' ' + i + ' ago';
                } else {
                    return counter + ' ' + i + 's ago';
                }
            }
        }
        return value;
    }
}
