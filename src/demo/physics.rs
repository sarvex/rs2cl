#[test]
use std::num::Zero;
#[test]
use nalgebra::traits::scalar_op::ScalarMul;
#[test]
use nalgebra::traits::dot::Dot;
#[test]
use kernel::Kernel;
#[test]
use cl_logic::ClOrd;
#[test]
use expr;
#[test]
use nalgebra2cl::*;
#[test]
use pragma;

// test
// #[test]
// fn integration_kernel() -> ()
// {
//   let k          = @mut Kernel::new(~"integrate");
// 
//   k.enable_extension(pragma::cl_khr_fp64);
// 
//   let velocities = k.named_param::<~[CLVec3f64]>(~"velocities", expr::Global);
//   let positions  = k.named_param::<~[CLVec3f64]>(~"positions", expr::Global);
//   let fext       = k.named_param::<CLVec3f64>(~"fext", expr::Global);
//   let dt         = k.named_param::<f64>(~"dt", expr::Global);
// 
//   // FIXME: let id         = k.global_id();
//   let id = expr::literal(0u32);
// 
//   velocities[id].assign(velocities[id] + fext.scalar_mul(&dt));
//   positions[id].assign(positions[id] + velocities[id].scalar_mul(&dt));
// 
//   println(k.to_str());
// }

#[test]
fn lin_pgs_kernel()
{
  let k = @mut Kernel::new(~"lin_pgs_solve");

  k.enable_extension(pragma::cl_khr_fp64);

  /*
   * Params
   */
  let id1s       = k.named_param::<~[u32]>(~"id1s", expr::Global);
  let id2s       = k.named_param::<~[u32]>(~"id2s", expr::Global);
  let normals    = k.named_param::<~[CLVec3f64]>(~"normals", expr::Global);
  let inv_masses = k.named_param::<~[f64]>(~"inv_masses", expr::Global);
  let impulses   = k.named_param::<~[f64]>(~"impulses", expr::Global);
  let lobounds   = k.named_param::<~[f64]>(~"lobounds", expr::Global);
  let hibounds   = k.named_param::<~[f64]>(~"hibounds", expr::Global);
  let objectives = k.named_param::<~[f64]>(~"objectives", expr::Global);
  let pmasses    = k.named_param::<~[f64]>(~"pmasses", expr::Global);
  let MJLambdas  = k.named_param::<~[CLVec3f64]>(~"MJLambdas", expr::Global);

  /*
   * Locals
   */
  let id         = k.named_var::<u32>(~"id");
  let d_lambda_i = k.named_var::<f64>(~"d_lambda_i");
  let id1        = k.named_var::<u32>(~"id1");
  let id2        = k.named_var::<u32>(~"id2");

  id.assign(k.get_global_id(0));

  id1.assign(id1s[id]);
  id2.assign(id2s[id]);

  /*
   * The solver itself
   */
  d_lambda_i.assign(objectives[id]);

  do k.if_(id1.cl_ge(&Zero::zero()))
  { d_lambda_i.assign(d_lambda_i + normals[id].dot(&MJLambdas[id1])); }

  do k.if_(id2.cl_ge(&Zero::zero()))
  { d_lambda_i.assign(d_lambda_i - normals[id].dot(&MJLambdas[id2])); }

  d_lambda_i.assign(d_lambda_i / pmasses[id]);

  let lambda_i_0 = k.var::<f64>();

  lambda_i_0.assign(impulses[id]);

  impulses[id].assign((lambda_i_0 + d_lambda_i).clamp(&lobounds[id], &hibounds[id]));

  d_lambda_i.assign(impulses[id] - lambda_i_0);

  do k.if_(id1.cl_ge(&Zero::zero()))
  { MJLambdas[id1].assign(MJLambdas[id1] - normals[id].scalar_mul(&(inv_masses[id1] * d_lambda_i))); }

  do k.if_(id2.cl_ge(&Zero::zero()))
  { MJLambdas[id2].assign(MJLambdas[id2] + normals[id].scalar_mul(&(inv_masses[id2] * d_lambda_i))); }

  println(k.to_str());
}
