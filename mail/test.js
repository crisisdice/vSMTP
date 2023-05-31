"use strict";
const nodemailer = require("nodemailer");

// async..await is not allowed in global scope, must use a wrapper
async function main() {
  let transporter = nodemailer.createTransport({
    host: "81-28-6-251.cloud-xip.com",
    port: 465,
    secure: true, // true for 465, false for other ports
    //ignoreTLS: ,
    // auth: {
    //   user: "root",
    //   pass: "root",
    // },
    logger: true,
    debug: true
  });

  console.log('transport created')

  // transporter.verify(function (error, success) {
  //   if (error) {
  //     console.log(error);
  //   } else {
  //     console.log("Server is ready to take our messages");
  //   }
  // });

  // send mail with defined transport object
  let info = await transporter.sendMail({
    from: 'alexanderdaily001@gmail.com', // sender address
    to: "reader@81-28-6-251.cloud-xip.com", // list of receivers
    subject: "2023-04-16", // Subject line
    text: "Hello world?", // plain text body
    html: "<b>Hello world?</b>", // html body
  });
  //
  console.log("Message sent: %s", info.messageId);
  // Message sent: <b658f8ca-6296-ccf4-8306-87d57a0b4321@example.com>

  // Preview only available when sending through an Ethereal account
  //console.log("Preview URL: %s", nodemailer.getTestMessageUrl(info));
  // Preview URL: https://ethereal.email/message/WaQKMgKddxQDoou...
}

main().catch(console.error).then(() => { return });
