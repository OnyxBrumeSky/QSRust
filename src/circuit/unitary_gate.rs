use num::{complex::Complex32, integer::Roots};
use nalgebra::DMatrix;
	
pub fn u(&mut self, matrice : &'a[Complex32], target) -> Option<&str>{
		
		let len = matrice.len();
		let n = len.sqrt() as usize;
		if n % 2 !=0 && n * n != len  {
			panic!("{}", "Error: Tried to apply a non unitary matrice".red());
		}
		let mat = DMatrix::from_row_slice(n, n, &matrice);
		let mat_t = mat.adjoint();
		let product = &mat_t * &mat;
		let identity = DMatrix::<Complex32>::identity(n, n);
		let tol = 1e-6f32;
    	let max_diff = product.iter()
        	.zip(identity.iter())
       	 	.map(|(a, b)| (*a - *b).norm())
       	 	.fold(0.0, f32::max);

		if max_diff > tol
		{
			panic!("{}","Error: tried to apply U gate that is not unitary.\n".red())
		} else {
			self.instructions.push(IStruct::U(matrice));
			Some(INSTRUCTION_ADDED)
		}
	}