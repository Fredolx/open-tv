import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { SetupComponent } from './setup/setup.component';

const routes: Routes = [
  {path: "", component: SetupComponent},
  // {path: "setup", component: SetupComponent},
 // {path: "settings", component: SettingsComponent},
 // {path: "", component: HomeComponent},
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule]
})
export class AppRoutingModule { }
