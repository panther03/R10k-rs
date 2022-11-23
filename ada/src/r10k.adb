With Ada.Text_IO; Use Ada.Text_IO;

procedure R10k is
   type PReg is record
      num: Natural;
      ready: Boolean;
   end record;

   function Print_PReg (P: PReg) return String is
   begin
      return "PR#" & Integer'Image(P.num) & (if P.ready then "+" else " ");
   end;

   type VRegType is (F, R);

   type VReg is record
      type : VRegType;
      num: Natural range 0 .. 4;
   end record;



   P : PReg := (3, true);
begin
   Ada.Text_IO.Put(Print_PReg(P));
end R10k;
