<details>
<summary>XSD contract: <code>ContactType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="ContactType">
		<xs:annotation>
			<xs:documentation>ContactType describes the structure of a contact's details.</xs:documentation>
		</xs:annotation>
		<xs:sequence>
			<xs:element ref="common:Name" minOccurs="0" maxOccurs="unbounded"/>
			<xs:element name="Department" type="common:TextType" minOccurs="0" maxOccurs="unbounded">
				<xs:annotation>
					<xs:documentation>Department is designation of the organisational structure by a linguistic expression, within which the contact person works.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:element name="Role" type="common:TextType" minOccurs="0" maxOccurs="unbounded">
				<xs:annotation>
					<xs:documentation>Role is the responsibility of the contact person with respect to the object for which this person is the contact.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:choice minOccurs="0" maxOccurs="unbounded">
				<xs:element name="Telephone" type="xs:string">
					<xs:annotation>
						<xs:documentation>Telephone holds the telephone number for the contact person.</xs:documentation>
					</xs:annotation>
				</xs:element>
				<xs:element name="Fax" type="xs:string">
					<xs:annotation>
						<xs:documentation>Fax holds the fax number for the contact person.</xs:documentation>
					</xs:annotation>
				</xs:element>
				<xs:element name="X400" type="xs:string">
					<xs:annotation>
						<xs:documentation>X400 holds the X.400 address for the contact person.</xs:documentation>
					</xs:annotation>
				</xs:element>
				<xs:element name="URI" type="xs:anyURI">
					<xs:annotation>
						<xs:documentation>URI holds an information URL for the contact person.</xs:documentation>
					</xs:annotation>
				</xs:element>
				<xs:element name="Email" type="xs:string">
					<xs:annotation>
						<xs:documentation>Email holds the email address for the contact person.</xs:documentation>
					</xs:annotation>
				</xs:element>
			</xs:choice>
		</xs:sequence>
		<xs:attribute name="id" type="common:IDType" use="optional">
			<xs:annotation>
				<xs:documentation>The id attribute is used to carry user id information for the contact.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
	</xs:complexType>
```

</details>
